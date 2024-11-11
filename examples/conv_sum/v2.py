import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
import random
import time
import os

current_seed = int(time.time())

INPUT_WIDTH = 128
INPUT_DEPTH = 64
FILTER_WIDTH = 3

FILTER_SIZE = FILTER_WIDTH * FILTER_WIDTH
SIM_THRESHOLD = INPUT_WIDTH * INPUT_DEPTH * FILTER_SIZE # Complete matrix convolution.
LINENO_BITLENGTH = (INPUT_WIDTH * INPUT_DEPTH).bit_length()
SRAM_DEPTH = 1 << LINENO_BITLENGTH

filter_given = [i for i in range(FILTER_SIZE)] # Can be changed as needed.


def generate_random_hex_file(path, filename, line_count):
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
                   'count': Port(UInt(32)),
                   'is_finish': Port(Int(1))}, 
        )
        self.steps = RegArray(UInt(32), 1)
        self.result = RegArray(Int(32), 1)
        
    @module.combinational
    def build(self, filter_to_use: RegArray):
        
        width = self.rdata.dtype.bits
        rdata, cnt, is_finish = self.pop_all_ports(False)
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
            with Condition(is_finish):
                finish()

class Driver(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user):
        
        i_input = RegArray(UInt(32), 1)    # i of the input
        j_input = RegArray(UInt(32), 1)    # j of the input
        i_filter = RegArray(UInt(32), 1)   # i of the filter
        j_filter = RegArray(UInt(32), 1)   # j of the filter
        cnt_conv = RegArray(UInt(32), 1)   # For conv count
        
        vi_input = i_input[0]
        vj_input = j_input[0]
        vi_filter = i_filter[0]
        vj_filter = j_filter[0]
        v_conv = cnt_conv[0]
                
        addr_base = (vi_input * UInt(32)(INPUT_WIDTH))[0:31].bitcast(UInt(32)) + vj_input
        addr_offest = (vi_filter * UInt(32)(INPUT_WIDTH))[0:31].bitcast(UInt(32)) + vj_filter
        addr = (addr_base + addr_offest)[0:LINENO_BITLENGTH-1].bitcast(Int(LINENO_BITLENGTH))
        
        is_finish = Int(1)(1)
        is_finish = (vi_input==UInt(32)(INPUT_DEPTH-FILTER_WIDTH)).select(is_finish, Int(1)(0))
        is_finish = (vj_input==UInt(32)(INPUT_WIDTH-FILTER_WIDTH)).select(is_finish, Int(1)(0))
        user.bind(count=v_conv, is_finish=is_finish)
        
        sram = SRAM(width, SRAM_DEPTH, init_file)
        sram.build(Int(1)(0), Int(1)(1), addr, Bits(width)(0), user)
        sram.bound.async_called()
        
        vi_filter = (vj_filter==UInt(32)(FILTER_WIDTH-1)).select(vi_filter+UInt(32)(1), vi_filter)
        vi_filter = (vi_filter==UInt(32)(FILTER_WIDTH)).select(UInt(32)(0), vi_filter)
        vj_filter = (vj_filter==UInt(32)(FILTER_WIDTH-1)).select(UInt(32)(0), vj_filter+UInt(32)(1))
        
        with Condition(v_conv==UInt(32)(FILTER_SIZE-1)):
            vi_input = (vj_input==UInt(32)(INPUT_WIDTH-FILTER_WIDTH)).select(vi_input+UInt(32)(1), vi_input)
            vj_input = (vj_input==UInt(32)(INPUT_WIDTH-FILTER_WIDTH)).select(UInt(32)(0), vj_input+UInt(32)(1))
            i_input[0] = vi_input
            j_input[0] = vj_input

        v_conv = (v_conv==UInt(32)(FILTER_SIZE-1)).select(UInt(32)(0), v_conv+UInt(32)(1))        
        
        i_filter[0] = vi_filter
        j_filter[0] = vj_filter
        cnt_conv[0] = v_conv

def check(raw, file_path):
    conv_sums = []
    for line in raw.splitlines():
        if 'Conv_sum:' in line:
            toks = line.split()
            conv_sums.append(int(toks[-1]))
    
    input_file = [[0] * INPUT_WIDTH for _ in range(INPUT_DEPTH)]
    with open(file_path, 'r') as file:
        row, col = 0, 0
        for line in file:
            input_file[row][col] = int(line.strip(), 16)
            col += 1
            if col == INPUT_WIDTH:
                col = 0
                row += 1
            if row == INPUT_DEPTH:
                break

    # Apply convolution without padding and compare with conv_sums
    filter_matrix = [[filter_given[i * FILTER_WIDTH + j] for j in range(FILTER_WIDTH)] for i in range(FILTER_WIDTH)]
    step = 0
    for i in range(INPUT_DEPTH - FILTER_WIDTH + 1):
        for j in range(INPUT_WIDTH - FILTER_WIDTH + 1):
            conv_result = 0
            for k in range(FILTER_WIDTH):
                for l in range(FILTER_WIDTH):
                    conv_result += input_file[i + k][j + l] * filter_matrix[k][l]
            
            if step < len(conv_sums):
                conv_sum = conv_sums[step]
                assert conv_sum == conv_result, f"Mismatch at step {step}: {conv_sum} != {conv_result}"

            step += 1

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

    config = backend.config(sim_threshold=SIM_THRESHOLD, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)
        
    file_path = os.path.join(resource_base, init_file)
       
    raw = utils.run_simulator(simulator_path)
    check(raw, file_path)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        check(raw, file_path)
        
    print(f"Seed is {current_seed}.") # For reproducing when problems occur

def test_convolution(sys_name, path, file):
    impl(sys_name, 32, file, path)

if __name__ == "__main__":
    sys_name = 'conv_sum'
    path = f'/tmp/{sys_name}'
    file = 'inputfile.hex'
    generate_random_hex_file(path, file, INPUT_WIDTH*INPUT_DEPTH)
    test_convolution(sys_name, path, file)

