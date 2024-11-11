import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils

starts = [0, 1, 2]

lineno_bitlength = 10
sram_depth = 1 << lineno_bitlength

filter_given = [1, 2, 3,
                4, 5, 6,
                7, 8, 9]

class MemUser(Module):

    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width)),
                   'count': Port(UInt(32))}, 
        )
        self.steps = RegArray(UInt(32), 1)
        self.result = RegArray(Int(32), 1)
        
    @module.combinational
    def build(self, filter_to_use: RegArray):
        
        width = self.rdata.dtype.bits
        rdata, cnt = self.pop_all_ports(False)
        rdata = rdata.bitcast(Int(width))
        
        filter_select = filter_to_use[cnt]

        unit_product = (rdata * filter_select)[0:31].bitcast(Int(32))
        
        conv_sum = self.result[0] + unit_product
        
        with Condition(cnt<UInt(32)(8)):
            self.result[0] = conv_sum
            
        with Condition(cnt==UInt(32)(8)):
            step = self.steps[0] + UInt(32)(1)
            log("Step: {}\tConv_sum: {}", step ,conv_sum)
            self.steps[0] = step
            self.result[0] = Int(32)(0)

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user):
        
        addr_start = RegArray(Int(lineno_bitlength), 3, initializer=starts)

        cnt_sram = RegArray(UInt(32), 1, initializer=[0])   # For sram traversal
        cnt_conv = RegArray(UInt(32), 1, initializer=[0])   # For conv resetting
        cnt_addr = RegArray(UInt(32), 1, initializer=[0])   # For addr increment
        
        v_sram = cnt_sram[0]
        v_conv = cnt_conv[0]
        v_addr = cnt_addr[0]

        addr_base = v_addr[0:lineno_bitlength-1].bitcast(Int(lineno_bitlength))
        addr = addr_base + addr_start[v_sram]
        
        user.bind(count=v_conv)
        
        sram = SRAM(width, sram_depth, init_file)
        sram.build(Int(1)(0), Int(1)(1), addr, Bits(width)(0), user)
        sram.bound.async_called()
        
        v_addr = (v_sram==UInt(32)(2)).select(v_addr+UInt(32)(1), v_addr)
        v_sram = (v_sram==UInt(32)(2)).select(UInt(32)(0), v_sram+UInt(32)(1))        
        
        v_addr = (v_conv==UInt(32)(8)).select(v_addr-UInt(32)(2), v_addr)
        v_conv = (v_conv==UInt(32)(8)).select(UInt(32)(0), v_conv+UInt(32)(1))
        
        cnt_sram[0] = v_sram
        cnt_conv[0] = v_conv
        cnt_addr[0] = v_addr

def check(raw):
    for line in raw.splitlines():
        if 'Conv_sum:' in line:
            toks = line.split()
            step = int(toks[-3])
            conv_sum = int(toks[-1])
            
            input = [start + step + i for start in starts for i in range(3)]
            
            result = sum(x * y for x, y in zip(input, filter_given))
            
            assert conv_sum == result, f"Mismatch at step {step}: conv_sum != result ({conv_sum} != {result})"

def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        
        filter_to_use = RegArray(
            scalar_ty=Int(32),
            size=9,
            initializer=filter_given
        )
        
        user = MemUser(width)
        user.build(filter_to_use)
        # Build the driver
        driver = Driver()
        driver.build(width, init_file, user)

    config = backend.config(sim_threshold=500, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        check(raw)

def test_convolution():
    impl('conv_filter', 32, 'init_1.hex', f'{utils.repo_path()}/python/unit-tests/resources')

if __name__ == "__main__":
        test_convolution()
