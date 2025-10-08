from assassyn.frontend import *
from assassyn import backend
from assassyn import utils

start_1 = 0
start_2 = 1
start_3 = 2

lineno_bitlength = 10
sram_depth = 1 << lineno_bitlength

filter_given = [1, 2, 3,
                4, 5, 6,
                7, 8, 9]

class ConvFilter(Downstream):

    def __init__(self):
        super().__init__()
        self.input = [[RegArray(Int(32), 1) for j in range(3)] for i in range(3)]
        self.ready = RegArray(UInt(32), 1)
        self.name = "Conv"

    @downstream.combinational
    def build(self, newly_loaded: Value, filter_to_use: RegArray):
        for i in range(3):
            for j in range(2):
                self.input[i][j][0] = self.input[i][j+1][0]
        for i in range(3):
            self.input[i][2][0] = newly_loaded[i]
        
        is_ready = self.ready[0]
        self.ready[0] = is_ready + UInt(32)(1)

        # The input is already full.
        with Condition(is_ready > UInt(32)(1)):                        
            # Convolution of two 3x3 matrices:
            # self.input    filter_to_use
            # [a b c]   *   [1 4 7]
            # [d e f]       [2 5 8]
            # [g h i]       [3 6 9]
            # Element-wise multiplication and summation for convolution result.
            conv_value = Int(32)(0)
            for i in range(3):
                for j in range(1,3):
                    unit_product = self.input[i][j][0] * filter_to_use[(j-1)*3+i]
                    conv_value = conv_value + unit_product[0:31].bitcast(Int(32))
            for i in range(3):
                unit_product = newly_loaded[i] * filter_to_use[6+i]
                conv_value = conv_value + unit_product[0:31].bitcast(Int(32))
                
            log("Step: {}\tConv_sum: {}", is_ready - UInt(32)(1) ,conv_value)

class ForwardData(Module):
    def __init__(self):
        super().__init__(
            ports={'rdata': Port(Bits(32))},
        ) 

    @module.combinational
    def build(self):
        data = self.pop_all_ports(True)
        data = data.bitcast(Int(32))
        return data

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user_1, user_2, user_3):

        cnt = RegArray(UInt(32), 1)
        v = cnt[0]

        addr_1 = v[0:lineno_bitlength-1].bitcast(Int(lineno_bitlength)) + Int(lineno_bitlength)(start_1)
        addr_2 = v[0:lineno_bitlength-1].bitcast(Int(lineno_bitlength)) + Int(lineno_bitlength)(start_2)
        addr_3 = v[0:lineno_bitlength-1].bitcast(Int(lineno_bitlength)) + Int(lineno_bitlength)(start_3)

        sram_1 = SRAM(width, sram_depth, init_file)
        sram_1.build(UInt(1)(0), UInt(1)(1), addr_1, Bits(width)(0))
        user_1.async_called()

        sram_2 = SRAM(width, sram_depth, init_file)
        sram_2.build(UInt(1)(0), UInt(1)(1), addr_2, Bits(width)(0))
        user_2.async_called()

        sram_3 = SRAM(width, sram_depth, init_file)
        sram_3.build(UInt(1)(0), UInt(1)(1), addr_3, Bits(width)(0))
        user_3.async_called()

        cnt[0] = v + UInt(32)(1)

def check(raw):
    for line in raw.splitlines():
        if 'Conv_sum:' in line:
            toks = line.split()
            step = int(toks[-3])
            conv_sum = int(toks[-1])
            
            input = [start_1+step, start_1+step+1 , start_1+step+2,
                     start_2+step, start_2+step+1 , start_2+step+2,
                     start_3+step, start_3+step+1 , start_3+step+2]

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
        
        user_1 = ForwardData()
        value_1 = user_1.build()

        user_2 = ForwardData()
        value_2 = user_2.build()

        user_3 = ForwardData()
        value_3 = user_3.build()

        driver = Driver()
        driver.build(width, init_file, user_1, user_2, user_3)

        data_load = [value_1, value_2, value_3]

        conv = ConvFilter()
        conv.build(data_load, filter_to_use)
        

    config = backend.config(sim_threshold=200, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        check(raw)

def test_filter():
    impl('conv_sum', 32, 'init_1.hex', f'{utils.repo_path()}/python/ci-tests/resources')

if __name__ == "__main__":
        test_filter()
