import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
import time
import random

current_seed = int(time.time())
current_seed = 1000

num_rows = 50
num_columns = 30
stride = 30

random.seed(current_seed)
num1 = random.randint(0, num_rows * num_columns)
num2 = random.randint(0, num_rows * num_columns)
start, end = sorted([num1, num2]) # Will get [start, end)

start_1 = 0
start_2 = 1
start_3 = 2

init_i  = start // num_columns
init_j = start % num_columns

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
        
        with Condition(is_ready > UInt(32)(1)):
            # log("INPUT: {} , {} , {} ", self.input[0][0][0], self.input[0][1][0], self.input[0][2][0])
            # log("INPUT: {} , {} , {} ", self.input[1][0][0], self.input[1][1][0], self.input[1][2][0])
            # log("INPUT: {} , {} , {} ", self.input[2][0][0], self.input[2][1][0], self.input[2][2][0])
            
            # log("INPUT: {} , {} , {} ", filter_to_use[0], filter_to_use[1], filter_to_use[2])
            # log("INPUT: {} , {} , {} ", filter_to_use[3], filter_to_use[4], filter_to_use[5])
            # log("INPUT: {} , {} , {} ", filter_to_use[6], filter_to_use[7], filter_to_use[8])
            
            
            # Convolution of two 3x3 matrices:
            # self.input    filter_to_use
            # [a b c]   *   [1 4 7]
            # [d e f]       [2 5 8]
            # [g h i]       [3 6 9]
            # Element-wise multiplication and summation for convolution result            
            conv_value = Int(32)(0)            
            for i in range(3):
                for j in range(1,3):
                    unit_product = self.input[i][j][0] * filter_to_use[(j-1)*3+i]
                    conv_value = conv_value + unit_product[0:31].bitcast(Int(32))
            for i in range(3):
                unit_product = newly_loaded[i] * filter_to_use[6+i]
                conv_value = conv_value + unit_product[0:31].bitcast(Int(32))
                
            log("Conv value: {}", conv_value)

class ForwardData(Module):
    def __init__(self):
        super().__init__(
            ports={'rdata': Port(Bits(32))},
        ) 

    @module.combinational
    def build(self):
        data = self.pop_all_ports(True)
        data = data.bitcast(Int(32))
        log("Forward: {}", data)
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
        sram_1.build(Int(1)(0), Int(1)(1), addr_1, Bits(width)(0), user_1)
        sram_1.bound.async_called()
        
        sram_2 = SRAM(width, sram_depth, init_file)
        sram_2.build(Int(1)(0), Int(1)(1), addr_2, Bits(width)(0), user_2)
        sram_2.bound.async_called()
        
        sram_3 = SRAM(width, sram_depth, init_file)
        sram_3.build(Int(1)(0), Int(1)(1), addr_3, Bits(width)(0), user_3)
        sram_3.bound.async_called()
        
        cnt[0] = v + UInt(32)(1)

# def check(raw):
#     for line in raw.splitlines():
#         if '[sram]' in line:
#             toks = line.split()
#             c = int(toks[-1])
#             b = int(toks[-3])
#             a = int(toks[-5])
#             assert a % 2 == 1 or c == 0, f'Expected odd number or zero, got {line}'
#             assert c == a + b, f'{a} + {b} = {c}'


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

        

    config = backend.config(sim_threshold=10, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    print(raw)
    # check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        # check(raw)

def test_filter():
    impl('conv_filter', 32, 'init_1.hex', f'{utils.repo_path()}/python/unit-tests/resources')

if __name__ == "__main__":
        test_filter()
