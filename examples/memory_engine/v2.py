import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
import time
import random

current_seed = int(time.time())

num_rows = 50
num_columns = 30
stride = 30

random.seed(current_seed)
num1 = random.randint(0, num_rows * num_columns)
num2 = random.randint(0, num_rows * num_columns)
start, end = sorted([num1, num2]) # Will get [start, end)

init_i  = start // num_columns
init_j = start % num_columns

cachesize = 8
lineno_bitlength = 11 # The maximum value of lineno is 50*30//8=187, but when the bitlength is set to 10, it will overflow
sram_depth = 1 << lineno_bitlength

class MemUser(Module):

    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width)),
                   'mask': Port(Bits(cachesize)),
                   'term': Port(Bits(1))
            }, 
        )
        
    @module.combinational
    def build(self):
        
        width = self.rdata.dtype.bits
        rdata, bitmask, term = self.pop_all_ports(False)

        data_joint = None

        for i in range(cachesize):
            offest = cachesize - i - 1
            data_masked = bitmask[offest:offest].select(rdata[offest*32:offest*32+31], Bits(32)(0))
            data_joint = data_masked if data_joint is None else data_joint.concat(data_masked)
        
        log("\tCacheline:\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            data_joint[224:255],
            data_joint[192:223],
            data_joint[160:191],
            data_joint[128:159],
            data_joint[96:127],
            data_joint[64:95],
            data_joint[32:63],
            data_joint[0:31])
        
        with Condition(term):
            finish()

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user):

        initialization = RegArray(UInt(1), 1)
        init = initialization[0]
        
        cnt_i = RegArray(Int(32), 1)
        cnt_j = RegArray(Int(32), 1)
        
        i = cnt_i[0]
        j = cnt_j[0]

        addr = (i * Int(32)(stride))[0:31].bitcast(Int(32)) + j
        row_end = (i * Int(32)(stride))[0:31].bitcast(Int(32)) + Int(32)(stride)
        shift = cachesize.bit_length() - 1
        
        # Initialization.
        with Condition(~(init)):
            initialization[0] = UInt(1)(1)
            cnt_i[0] = Int(32)(init_i)
            cnt_j[0] = Int(32)(init_j)
            
            log("start:{} end:{} i:{} j:{} seed:{}", Int(32)(start), Int(32)(end), Int(32)(init_i), Int(32)(init_j), Int(32)(current_seed))
        
        # i and j have already been initialized.
        with Condition(init):
            lineno = addr[shift:shift+lineno_bitlength-1].bitcast(UInt(lineno_bitlength))
            line_end = (Bits(32)(0).concat((lineno + UInt(lineno_bitlength)(1)) << UInt(lineno_bitlength)(cachesize.bit_length()-1)))[0:31].bitcast(Int(32))
            offset = Bits(cachesize-shift)(0).concat(addr[0:shift-1]).bitcast(Bits(cachesize))
            reserve = Bits(cachesize)((1<<cachesize) - 1) >> offset

            sram = SRAM(width, sram_depth, init_file)
            sram.build(UInt(1)(0), init, lineno, Bits(width)(0))
            
            sentinel = (Int(32)(end) <= row_end).select(Int(32)(end), row_end)
            nextrow = (Int(32)(end) <= row_end).select(UInt(1)(0), UInt(1)(1))
            
            counter = (line_end >= sentinel).select((line_end - sentinel).bitcast(UInt(32)), UInt(32)(0))
            discard = (UInt(cachesize)(1) << counter) - UInt(cachesize)(1)
            
            bitmask = (reserve ^ discard).bitcast(Bits(cachesize))
            
            simu_term = (line_end >= sentinel).select(UInt(1)(1), UInt(1)(0))
            simu_term = simu_term & ~nextrow
            
            log("\t\tCALL: bitmask={:b}\tlineno={}", bitmask, lineno)
            
            user.bind(mask=bitmask, term=simu_term)
            user.async_called()
            
            with Condition(line_end >= sentinel):
                # Read will go to next row.
                with Condition(nextrow):
                    cnt_i[0] = i + Int(32)(1)
                    cnt_j[0] = Int(32)(0)
                
            with Condition(line_end < sentinel):
                cnt_j[0] = j + Int(32)(cachesize) - (Bits(32-cachesize)(0).concat(offset)).bitcast(Int(32))

def check(raw):
    cache_values = []
    for line in raw.splitlines():
        if 'Cacheline:' in line:
            toks = line.split()
            values = toks[-8:]
            cache_values.extend(int(value) for value in values if int(value) != 0)
    
    # Verify if `cached_values` are a continuous array
    if cache_values:
        expected_values = list(range(start+1, end+1))
        # If `cached_values` are not equal to a continuous expected array, an error is reported
        if cache_values != expected_values:
            missing_values = set(expected_values) - set(cache_values)
            extra_values = set(cache_values) - set(expected_values)
            error_msg = (
                f"Error: Cacheline values are not continuous from {start} to {end}.\n"
                f"Missing values: {sorted(missing_values)}\n"
                f"Extra values: {sorted(extra_values)}"
            )
            assert False, error_msg

def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        user = MemUser(width)
        user.build()
        # Build the driver
        driver = Driver()
        driver.build(width, init_file, user)

    config = backend.config(sim_threshold=300, idle_threshold=50, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        check(raw)

def test_memory():
    impl('memory_init', 32*cachesize, 'init_8.hex', f'{utils.repo_path()}/python/ci-tests/resources')

if __name__ == "__main__":
        test_memory()
