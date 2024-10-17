#!/usr/bin/env python3

# This file aims at reading a objdump file and extracting the `.text` and `.data` sections for our CPU execution.

import os
import sys
import argparse
import subprocess

parser = argparse.ArgumentParser(description='Extract the `.text` and `.data` sections from the objdump file.')
parser.add_argument('--fname', type=str, help='The file name a objdump output.', required=True)
parser.add_argument('--odir', type=str, help='The output directory of the converted file.', default='.')
args = vars(parser.parse_args())

fname = args['fname']

print(f'Extracting {fname}...')

offset = 0x80000000
data_offset = 0

readelf = subprocess.check_output(['riscv64-unknown-elf-readelf', '-S', args['fname'][:-5]]).decode('utf-8')
readelf = readelf.split('\n')
for i in readelf:
    toks = i.strip().split()
    if not toks:
        continue
    if toks[0] == '[':
        toks[0] = toks[0] + toks[1]
        toks = [toks[0]] + toks[2:]
    n = len(toks)
    if n > 1 and toks[1] == '.data':
        data_offset = int(toks[3], 16)
    if n > 0 and toks[0] == '[1]':
        offset = int(toks[3], 16)

data_offset = data_offset - offset
assert data_offset % 4 == 0

bin_name = os.path.split(fname)[-1]

with open(args['odir'] + '/' + bin_name[:-10] + 'config', 'w') as f:
    f.write(f'{{ "offset": {hex(offset)}, "data_offset": {hex(data_offset)} }}')

text, data = [], []

section = ''
func = ''
func_tagged = False

with open(fname) as f:
    raw = f.readlines()
    for line in raw:
        toks = line.strip().split()
        n = len(toks)
        # Parse the section line
        if n == 4 and toks[0] == 'Disassembly' and toks[1] == 'of' and toks[2] == 'section':
            section = toks[-1]
            func = None
        # Parse the symbol line, it can be either a function or a array id
        elif n == 2 and toks[1].startswith('<') and toks[1].endswith('>:'):
            func = toks[1]
            func_tagged = False
        # If we are in a general `.text` section, including all sections named `.text*`,
        # like `text.startup`, `text.init`, etc. Load them into the text buffer.
        elif section.startswith('.text') and n >= 2:
            addr = int(toks[0][:-1], 16)
            assert addr >= offset and addr % 4 == 0
            comment = ' '.join(toks[2:])
            if not func_tagged:
                assert func
                comment = comment + ' ' + func
                func_tagged = True
            text.append((addr - offset, int(toks[1], 16), comment))
        # If we are in a `.data` section, load the data into the data buffer.
        elif section.startswith('.data') and n >= 2:
            addr = int(toks[0][:-1], 16)
            assert addr % 2 == 0, addr

            def append(addr, i, value):
                global func_tagged
                comment = func if not func_tagged else ''
                if comment:
                    assert (addr + 2 * i) % 4 == 0, 'Arrays should be 4-aligned'
                func_tagged = True
                data.append((addr + 2 * i, int(value, 16), comment))

            # This part is a little bit tricky. The dumped log looks something like:
            #
            # hex-addr [spaces] payload [riscv reinterpret]
            #
            # The problem is that the payload can be either 16-bit int, 32-bit int, or several 16-bit ints.
            # To robustly handle all these, I developeed a unified interface, handle all these in 16-bit
            # format first, and then coalesce them into 32-bit format later.
            addr = addr - offset

            realign = []
            for i in line:
                if i != '\t':
                    realign.append(i)
                else:
                    while len(realign) % 8 != 0:
                        realign.append(' ')
            realign = ''.join(realign)


            payload = realign[:40].split()[1:]

            for i, value in enumerate(payload):
                if len(value) == 4:
                    append(addr, i, value)
                elif len(value) == 8:
                    append(addr, 2 * i, value[4:])
                    append(addr, 2 * i + 1, value[:4])
                else:
                    assert False, f'TODO: len={len(value)} {line}'

        elif len(toks) > 1:
            pass

text.sort()
data.sort()

zero_padding = '0 // padding'

# Test section is easy, just dump all the loaded instructions, which are 4-byte aligned.
buffer = [zero_padding] * (text[-1][0] // 4 + 1)
for addr, inst, comment in text:
    inst = hex(inst)
    inst = (10 - len(inst)) * '0' + inst[2:]
    buffer[addr // 4] = inst + ' // ' + comment

assert bin_name.endswith('.riscv.dump')
ofile = args['odir'] + '/' + bin_name[:-10]
with open(ofile + 'exe', 'w') as f:
    f.write('\n'.join(buffer))

if data:
    # This is tricky to coalesce the data section from 2-byte to 4-byte.
    # We do this 2-byte by 2-byte.
    coalesced = []
    i = 0
    n = len(data)
    while i < len(data):
        residue = data[i][0] % 4
        # If it is aligned by 4, it is a start of coalescing.
        if residue == 0:
            # If it has following data to coalesce, combine those two into one.
            if i + 1 < n and data[i][0] + 2 == data[i + 1][0]:
                coalesced.append((data[i][0], data[i + 1][1] << 16 | data[i][1], data[i][2]))
            # If not, the high 16-bit is already implicitly zero, so just leave it.
            else:
                coalesced.append(data[i])
            i += 2
        else:
            # If it is not aligned by 4, it is the high 16-bit of coalescing. We pad the low 16-bit with zero.
            coalesced.append((data[i][0] - residue, data[i + 1][1] << 16, data[i][2]))
            i += 1
    
    buffer = [zero_padding] * (coalesced[-1][0] // 4 + 1)
    for addr, value, comment in coalesced:
        buffer[addr // 4] = hex(value)[2:] + ' // ' + comment + hex(addr)
    
    with open(ofile + 'data', 'w') as f:
        f.write('\n'.join(buffer[data_offset // 4:]))
