#!/usr/bin/env python3

# This file aims at reading a objdump file and extracting the `.text` and `.data` sections for our CPU execution.

import os
import sys
import argparse
import subprocess

parser = argparse.ArgumentParser(description='Extract the `.text` and `.data` sections from the objdump file.')
parser.add_argument('--fname', type=str, help='The file name a objdump output.', required=True)
parser.add_argument('--odir', type=str, help='The output directory of the converted file.', default='.')
parser.add_argument('--exit-tohost', action='store_true', help='Exit to host')
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

if bin_name.endswith('.riscv.dump'):
    strip_bin = bin_name[:-10]
elif bin_name.endswith('.dump'):
    strip_bin = bin_name[:-4]
else:
    strip_bin = bin_name

with open(args['odir'] + '/' + strip_bin + 'config', 'w') as f:
    f.write(f'{{ "offset": {hex(offset)}, "data_offset": {hex(data_offset)} }}')

text, data = [], []

section = ''
func = ''
func_tagged = False

def parse_payload(line):
    global func, func_tagged
    # Each line uses \t to balance the visual effect, so we need to realign the lines
    # by replacing them with spaces.
    realign = []
    for i in line:
        if i != '\t':
            realign.append(i)
        elif len(realign) % 8 == 0:
            realign.append(' ' * 8)
        else:
            while len(realign) % 8 != 0:
                realign.append(' ')
    realign = ''.join(realign)

    # After 40 characters, they are all semantic comments, so we ignore for payload.
    toks = realign[:40].split()
    # But retain for readability hints
    comment = realign[40:]

    addr = int(toks[0][:-1], 16) - offset

    payload = toks[1:]

    res = []

    j = 0
    for value in payload:
        if len(value) == 4:
            res.append([addr + 2 * j, int(value, 16), ''])
            j += 1
        elif len(value) == 8:
            res.append([addr + 2 * j, int(value[4:], 16), ''])
            res.append([addr + 2 * j + 2, int(value[:4], 16), ''])
            j += 2
        else:
            try:
                int(value)
                assert False, f'TODO: len={len(value)} {line}'
            except:
                # TODO(@were): Fix unaligned addresses for byte-level operations.
                print(value, 'is heuristically skipped')
                continue

    if not func_tagged:
        comment = comment + ' ' + func
        func_tagged = True

    if res:
        res[-1][-1] = comment

    return res

with open(fname) as f:
    raw = f.readlines()
    for line in raw:
        line = line.strip()
        toks = line.split()
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
            delta = parse_payload(line)
            text += delta 
        # If we are in a `.data` section, load the data into the data buffer.
        elif section.startswith('.data') and n >= 2:
            delta = parse_payload(line)
            data += delta
        elif len(toks) > 1:
            print(line)

text.sort()
data.sort()

zero_padding = '0 // padding'

def coalesce_words(a):
    res = []
    i = 0
    n = len(a)
    while i < len(a):
        residue = a[i][0] % 4
        # If it is aligned by 4, it is a start of coalescing.
        if residue == 0:
            # If it has following data to coalesce, combine those two into one.
            if i + 1 < n and a[i][0] + 2 == a[i + 1][0]:
                res.append((a[i][0], a[i + 1][1] << 16 | a[i][1], a[i][2] + ' | ' + a[i + 1][2]))
            # If not, the high 16-bit is already implicitly zero, so just leave it.
            else:
                res.append(a[i])
            i += 2
        else:
            # If it is not aligned by 4, it is the high 16-bit of coalescing. We pad the low 16-bit with zero.
            res.append((a[i][0] - residue, a[i][1] << 16, a[i][2]))
            i += 1

    uniq_n = len(set(i for i, _, _ in res))
    n = len(res)
    assert uniq_n == n, f'{uniq_n} {n}'
    return res


# Test section is easy, just dump all the loaded instructions, which are 4-byte aligned.

assert text
coalesced = coalesce_words(text)

assert coalesced[-1][0] // 4 + 1 < 16000, f'{coalesced[-1][0] // 4 + 1}'

buffer = [zero_padding] * (coalesced[-1][0] // 4 + 1)

for addr, inst, comment in coalesced:
    inst = hex(inst)
    inst = (10 - len(inst)) * '0' + inst[2:]
    buffer[addr // 4] = inst + ' // ' + comment


ofile = args['odir'] + '/' + strip_bin
with open(ofile + 'exe', 'w') as f:
    f.write('\n'.join(buffer))

if data:
    # This is tricky to coalesce the data section from 2-byte to 4-byte.
    # We do this 2-byte by 2-byte.
    coalesced = coalesce_words(data)

    assert coalesced[-1][0] // 4 + 1 < 160000

    buffer = [zero_padding] * (coalesced[-1][0] // 4 + 1)
    for addr, value, comment in coalesced:
        assert addr % 4 == 0
        value = hex(value)[2:]
        # value = (10 - len(value)) * '0' + value[2:]
        buffer[addr // 4] = value + ' // ' + comment + hex(addr)
    
    with open(ofile + 'data', 'w') as f:
        f.write('\n'.join(buffer[data_offset // 4:]))
