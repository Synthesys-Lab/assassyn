import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
import time
import random

current_seed = int(time.time())
current_seed = 1000

cachesize = 8
bitmask = 0b10101010

num_rows = 50
num_columns = 30
stride = 30

random.seed(current_seed)
num1 = random.randint(0, num_rows * num_columns)
num2 = random.randint(0, num_rows * num_columns)
start, end = sorted([num1, num2]) # Will get [start, end)

start = 5
end = 33

init_i  = start // num_columns
init_j = start % num_columns

class MemUser(Module):

    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width)),
                   'mask': Port(Bits(cachesize))
            }, 
        )
        self.reg_accm = RegArray(Int(width), 1)
        self.name = "sram"
        
    @module.combinational
    def build(self):
        
        width = self.rdata.dtype.bits
        rdata, bitmask = self.pop_all_ports(False)
        rdata = rdata.bitcast(Int(width))

        data_joint = None

        for i in range(cachesize):
            offest = cachesize - i - 1
            data_masked = bitmask[offest:offest].select(rdata[offest*32:offest*32+31], Bits(32)(0))
            data_joint = data_masked if data_joint is None else data_joint.concat(data_masked)

        log("Cacheline:\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            data_joint[224:255],
            data_joint[192:223],
            data_joint[160:191],
            data_joint[128:159],
            data_joint[96:127],
            data_joint[64:95],
            data_joint[32:63],
            data_joint[0:31])

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user):

        initialization = RegArray(Int(1), 1)
        init = initialization[0]
        
        cnt_i = RegArray(Int(32), 1)
        cnt_j = RegArray(Int(32), 1)
        
        # Initialization.
        with Condition(~init):
            initialization[0] = Int(1)(1)
            cnt_i[0] = Int(32)(init_i)
            cnt_j[0] = Int(32)(init_j)
            
            log("start:{} end:{} i:{} j:{}", Int(32)(start), Int(32)(end), Int(32)(init_i), Int(32)(init_j))
        
        # i and j have already been initialized.
        with Condition(init):
        
            i = cnt_i[0]
            j = cnt_j[0]

            addr = (i * Int(32)(stride))[0:31].bitcast(Int(32)) + j
            row_end = (i * Int(32)(stride))[0:31].bitcast(Int(32)) + Int(32)(stride)
            shift = cachesize.bit_length() - 1
            lineno = Bits(shift)(0).concat(addr[shift:8]).bitcast(Int(9))
            line_end = ((lineno +Int(9)(1))*Int(32)(cachesize))[0:31].bitcast(Int(32))
            offset = Bits(cachesize-shift)(0).concat(addr[0:shift-1]).bitcast(Bits(cachesize))
            reserve = Bits(cachesize)(2 ** cachesize - 1) >> offset

            log("i={}\tj={}\taddr={}\trow_end={}\tlineno={}\tline_end={}\toffest={}\treserve={:b}", i, j, addr, row_end, lineno, line_end, offset, reserve)
            
            # sram = SRAM(width, 512, init_file)
            # sram.build(Int(1)(0), Int(1)(1), lineno, Bits(width)(0), user)
            
            sentinel = (Int(32)(end) <= row_end).select(Int(32)(end), row_end)
            next = (Int(32)(end) <= row_end).select(Int(1)(0), Int(1)(1))
            
            with Condition(line_end >= sentinel):
                discard = (UInt(cachesize)(1) << ((line_end - sentinel).bitcast(UInt(32)))) - UInt(cachesize)(1)
                bitmask = reserve ^ discard
                log("get end: reserve={:b} discard={:b} bitmask={:b}", reserve, discard, bitmask)
                
                # Read will go to next row.
                with Condition(next):
                    cnt_i[0] = i + Int(32)(1)
                    cnt_j[0] = Int(32)(0)
                # Read will finish in current row.                
                with Condition(~next): 
                    initialization[0] = Int(1)(0)
                
            with Condition(line_end < sentinel):
                bitmask = reserve
                log("full read 1: bitmask is {:b}", bitmask)
                cnt_j[0] = j + Int(32)(cachesize) - (Bits(32-cachesize)(0).concat(offset)).bitcast(Int(32))
            

        # raddr = v[0:8].bitcast(Int(9))
        
        # shift = Int(9)(cachesize.bit_length())
        # addr_access = raddr >> shift
        
        # mask = Bits(cachesize)(bitmask)
        # user.bind(mask=mask)
        
        # self.cnt[0] = plused
        # sram = SRAM(width, 512, init_file)
        # sram.build(we, re, addr_access, v.bitcast(Bits(width)), user)
        # with Condition(re):
        #     sram.bound.async_called()

def check(raw):
    for line in raw.splitlines():
        if '[sram]' in line:
            toks = line.split()
            c = int(toks[-1])
            b = int(toks[-3])
            a = int(toks[-5])
            assert a % 2 == 1 or c == 0, f'Expected odd number or zero, got {line}'
            assert c == a + b, f'{a} + {b} = {c}'


def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        user = MemUser(width)
        user.build()
        # Build the driver
        driver = Driver()
        driver.build(width, init_file, user)

    config = backend.config(sim_threshold=200, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    print(raw)
    # check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        # check(raw)

def test_memory():
    impl('memory_init', 32*cachesize, 'init_2.hex', f'{utils.repo_path()}/python/unit-tests/resources')

if __name__ == "__main__":
        test_memory()
