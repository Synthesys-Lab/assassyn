import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
import random
import time
import os

# current_seed = int(time.time())
current_seed = 1000

INPUT_WIDTH = 128
INPUT_DEPTH = 64

FILTER_WIDTH = 3
FILTER_SIZE = FILTER_WIDTH * FILTER_WIDTH

filter_given = [i for i in range(FILTER_SIZE)] # Can be changed as needed.

lineno_bitlength = 20
sram_depth = 1 << lineno_bitlength


def generate_random_hex_file(path, filename, line_count):
    """
    Generates a file with each line containing a random hexadecimal number between 0 and 255.
    
    Parameters:
    - path (str): The directory path where the file will be saved. If it doesn't exist, it will be created.
    - filename (str): The name of the file to be created.
    - line_count (int): The number of lines in the file.
    """
    # Check if the path exists; if not, create it
    if not os.path.exists(path):
        os.makedirs(path)

    random.seed(current_seed)
    # Generate random hex numbers and write to the file
    file_path = os.path.join(path, filename)
    with open(file_path, 'w') as file:
        for _ in range(line_count):
            random_number = random.randint(0, 255)
            file.write(f"{random_number:08X}\n")
    
    print(f"File generated at: {file_path}")


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
        
        with Condition(cnt<UInt(32)(FILTER_SIZE-1)):
            self.result[0] = conv_sum
            
        with Condition(cnt==UInt(32)(FILTER_SIZE-1)):
            step = self.steps[0] + UInt(32)(1)
            log("Step: {}\tConv_sum: {}", step ,conv_sum)
            self.steps[0] = step
            self.result[0] = Int(32)(0)

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user):
        
        input_i = RegArray(UInt(32), 1, initializer=[0])
        input_j = RegArray(UInt(32), 1, initializer=[0])
        
        vi_input = input_i[0]
        vj_input = input_j[0]
        
        addr_ij = (vi_input * UInt(32)(INPUT_WIDTH))[0:31].bitcast(UInt(32)) + vj_input
        
        sram_starts = [i for i in range(FILTER_WIDTH)]
        addr_start = RegArray(Int(lineno_bitlength), FILTER_WIDTH, initializer=sram_starts)

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
        
        v_addr = (v_sram==UInt(32)(FILTER_WIDTH-1)).select(v_addr+UInt(32)(1), v_addr)
        v_sram = (v_sram==UInt(32)(FILTER_WIDTH-1)).select(UInt(32)(0), v_sram+UInt(32)(1))        

        v_addr = (v_conv==UInt(32)(FILTER_SIZE-1)).select(v_addr-UInt(32)(FILTER_WIDTH-1), v_addr)
        v_conv = (v_conv==UInt(32)(FILTER_SIZE-1)).select(UInt(32)(0), v_conv+UInt(32)(1))
        
        cnt_sram[0] = v_sram
        cnt_conv[0] = v_conv
        cnt_addr[0] = v_addr

def check(raw, file_path):
            
    input_file = []
    with open(file_path, 'r') as file:
        for line in file:
            input_file.append(int(line.strip(), 16))            
    # print(input_file[0:9])
    
    for line in raw.splitlines():
        if 'Conv_sum:' in line:
            toks = line.split()
            step = int(toks[-3])
            conv_sum = int(toks[-1])
            
            input = [input_file[start+step+i-1] for start in [j for j in range(FILTER_WIDTH)] for i in range(FILTER_WIDTH)]
            
            result = sum(x * y for x, y in zip(input, filter_given))
            
            assert conv_sum == result, f"Mismatch at step {step}: conv_sum != result ({conv_sum} != {result})"

def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        
        filter_to_use = RegArray(
            scalar_ty=Int(32),
            size=FILTER_SIZE,
            initializer=filter_given
        )
        
        user = MemUser(width)
        user.build(filter_to_use)
        # Build the driver
        driver = Driver()
        driver.build(width, init_file, user)

    config = backend.config(sim_threshold=500, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)
        
    print(f"Seed is {current_seed}.") # For reproducing when problems occur
    file_path = os.path.join(resource_base, init_file)
       
    raw = utils.run_simulator(simulator_path)
    print(raw)
    check(raw, file_path)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        check(raw, file_path)

def test_convolution(sys_name, path, file):
    impl(sys_name, 32, file, path)
    # impl('conv_sum', 32, 'init_1.hex', f'{utils.repo_path()}/python/unit-tests/resources')

if __name__ == "__main__":
    sys_name = 'conv_sum'
    path = f'/tmp/{sys_name}'
    file = 'inputfile.hex'
    generate_random_hex_file(path, file, INPUT_WIDTH*INPUT_DEPTH)
    test_convolution(sys_name, path, file)

