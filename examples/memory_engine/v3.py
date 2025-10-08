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
step = random.randint(1, 8)

init_i  = start // num_columns
init_j = start % num_columns

cachesize = 8
lineno_bitlength = 11 # The maximum value of lineno is 50*30//8=187, but when the bitlength is set to 10, it will overflow
sram_depth = 1 << lineno_bitlength

class MemMontage(Module):
    def __init__(self, width):
        super().__init__(
            ports={'ldata': Port(Bits(width)),
                   'rdata': Port(Bits(width)),
                   'mask': Port(UInt(8)),
                   'term': Port(Bits(1))
                   }, 
        )
        
    @module.combinational
    def build(self):
        
        width = self.ldata.dtype.bits
        width2 = width<<1
        ldata, rdata, mask, term = self.pop_all_ports(False)
        ldata = Bits(width)(0).concat(ldata).bitcast(Int(width2))
        rdata = Bits(width)(0).concat(rdata).bitcast(Int(width2))

        data_joint = (Bits(width2)(0) | ldata).bitcast(UInt(width2))
        data_joint = data_joint << mask
        data_joint = data_joint | rdata

        log("Cacheline:\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
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

class MemUser(Module):

    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width)),
                   'mask': Port(Bits(cachesize)),
                   'term': Port(Bits(1))
            }, 
        )
        
    @module.combinational
    def build(self, montage):
        
        width = self.rdata.dtype.bits
        rdata, bitmask, term = self.pop_all_ports(False)

        data1 = bitmask[7:7].select(Bits(32)(0).concat(rdata[224:255]).bitcast(UInt(64)), UInt(64)(0))
        data2 = bitmask[6:6].select(Bits(32)(0).concat(rdata[192:223]).bitcast(UInt(64)), UInt(64)(0))
        data3 = bitmask[5:5].select(Bits(32)(0).concat(rdata[160:191]).bitcast(UInt(64)), UInt(64)(0))
        data4 = bitmask[4:4].select(Bits(32)(0).concat(rdata[128:159]).bitcast(UInt(64)), UInt(64)(0))
        data5 = bitmask[3:3].select(Bits(32)(0).concat(rdata[96:127]).bitcast(UInt(64)), UInt(64)(0))
        data6 = bitmask[2:2].select(Bits(32)(0).concat(rdata[64:95]).bitcast(UInt(64)), UInt(64)(0))
        data7 = bitmask[1:1].select(Bits(32)(0).concat(rdata[32:63]).bitcast(UInt(64)), UInt(64)(0))
        data8 = bitmask[0:0].select(Bits(32)(0).concat(rdata[0:31]).bitcast(UInt(64)), UInt(64)(0))

        mask2 = bitmask[6:6].select(UInt(8)(32), UInt(8)(0))
        mask3 = bitmask[5:5].select(UInt(8)(32), UInt(8)(0))
        mask4 = bitmask[4:4].select(UInt(8)(32), UInt(8)(0))
        mask5 = bitmask[3:3].select(UInt(8)(32), UInt(8)(0))
        mask6 = bitmask[2:2].select(UInt(8)(32), UInt(8)(0))
        mask7 = bitmask[1:1].select(UInt(8)(32), UInt(8)(0))
        mask8 = bitmask[0:0].select(UInt(8)(32), UInt(8)(0))
        
        data12 = (((UInt(64)(0) | data1).bitcast(UInt(64))) << mask2) | data2
        data12 = (UInt(64)(0)).concat(data12).bitcast(UInt(128))
        
        data34 = (((UInt(64)(0) | data3).bitcast(UInt(64))) << mask4) | data4
        data34 = (UInt(64)(0)).concat(data34).bitcast(UInt(128))
        
        data56 = (((UInt(64)(0) | data5).bitcast(UInt(64))) << mask6) | data6
        data56 = (UInt(64)(0)).concat(data56).bitcast(UInt(128))
        
        data78 = (((UInt(64)(0) | data7).bitcast(UInt(64))) << mask8) | data8
        data78 = (UInt(64)(0)).concat(data78).bitcast(UInt(128))        
        
        mask34 = mask3 + mask4
        mask56 = mask5 + mask6
        mask78 = mask7 + mask8
        
        data1234 = (((UInt(128)(0) | data12).bitcast(UInt(128))) << mask34) | data34
        data5678 = (((UInt(128)(0) | data56).bitcast(UInt(128))) << mask78) | data78
        
        mask5678 = mask56 + mask78
        
        montage.async_called(ldata=data1234, rdata=data5678, mask=mask5678, term=term)


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
        
        lineno = addr[shift:shift+lineno_bitlength-1].bitcast(UInt(lineno_bitlength))
        line_start = (Bits(32)(0).concat(lineno << UInt(lineno_bitlength)(cachesize.bit_length()-1)))[0:31].bitcast(Int(32))   
        line_end = line_start + Int(32)(cachesize)     
                
        start_addr = (Int(32)(init_i) * Int(32)(stride))[0:31].bitcast(Int(32)) + Int(32)(init_j)
        line0_addr = start_addr[shift:31].concat(Int(shift)(0)).bitcast(Int(32))
        adjust1 = (start_addr - line0_addr) % Int(32)(step)
        adjust2 = Int(32)(step) - ((line_start - line0_addr) % Int(32)(step))
        perturbation = ((adjust1 + adjust2) % Int(32)(step))[0:cachesize-1].bitcast(Bits(cachesize))
                
        # Initialization.
        with Condition(~(init)):
            initialization[0] = UInt(1)(1)
            cnt_i[0] = Int(32)(init_i)
            cnt_j[0] = Int(32)(init_j)
            
            log("seed:{}\tstart:{}\tend:{}\tstep:{}\ti:{}\tj:{}", Int(32)(current_seed), Int(32)(start), Int(32)(end), Int(32)(step), Int(32)(init_i), Int(32)(init_j))
        
        # i and j have already been initialized.
        with Condition(init):
            
            # When Cachesize=8
            kstep = (Int(8)(step)<Int(8)(9)).select(UInt(8)(step-1), UInt(8)(0)) 
            maskselect = UInt(8)(1) << kstep
            maskinit = maskselect.select1hot(Bits(8)(0b11111111),
                                             Bits(8)(0b10101010),
                                             Bits(8)(0b10010010),
                                             Bits(8)(0b10001000),
                                             Bits(8)(0b10000100),
                                             Bits(8)(0b10000010),
                                             Bits(8)(0b10000001),
                                             Bits(8)(0b10000000))
            
            maskinit = maskinit >> perturbation
            
            offset = Bits(cachesize-shift)(0).concat(addr[0:shift-1]).bitcast(UInt(cachesize))
            remove = ((UInt(cachesize+1)(1) << (UInt(cachesize)(cachesize)-offset)) - UInt(cachesize)(1))[0:cachesize-1]
            reserve = maskinit & remove # Remove those data before start in this cacheline.

            sram = SRAM(width, sram_depth, init_file)
            sram.build(UInt(1)(0), init, lineno, Bits(width)(0), user)
            
            sentinel = (Int(32)(end) <= row_end).select(Int(32)(end), row_end)
            nextrow = (Int(32)(end) <= row_end).select(UInt(1)(0), UInt(1)(1))
            
            counter = (line_end >= sentinel).select((line_end - sentinel).bitcast(UInt(32)), UInt(32)(0))
            discard = ~((UInt(cachesize)(1) << counter) - UInt(cachesize)(1))
            
            bitmask = reserve & discard # Discard those data after end in this cacheline.
            
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
    
    # Verify if `cache_values` are a continuous array with the specified step
    if cache_values:
        expected_values = list(range(start+1, end+1, step))
        
        # If `cache_values` are not equal to a continuous expected array, report an error
        if cache_values != expected_values:
            missing_values = set(expected_values) - set(cache_values)
            extra_values = set(cache_values) - set(expected_values)
            error_msg = (
                f"Error: Cacheline values are not continuous from {start} to {end} with step {step}.\n"
                f"Missing values: {sorted(missing_values)}\n"
                f"Extra values: {sorted(extra_values)}"
            )
            assert False, error_msg

def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        montage = MemMontage(width>>1)
        montage.build()
        
        user = MemUser(width)
        user.build(montage)

        # Build the driver
        driver = Driver()
        driver.build(width, init_file, user)

    config = backend.config(sim_threshold=300, idle_threshold=50, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    # print(raw)
    check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        check(raw)

def test_memory():
    impl('memory_init', 32*cachesize, 'init_8.hex', f'{utils.repo_path()}/python/ci-tests/resources')

if __name__ == "__main__":
        test_memory()
