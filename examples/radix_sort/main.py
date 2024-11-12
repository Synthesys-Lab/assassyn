# Radix sort
# 2 stage machine
# Stage 1: read data from memory and put them into a register array based on the radix
# Stage 2: prefix sum the radix
# Stage 3: write data to memory
import os

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

# Resource base path
current_path = os.path.dirname(os.path.abspath(__file__))
resource_base = f'{current_path}/workload/'
print(f'resource_base: {resource_base}')
# Data width, length
data_width = 32
data_depth = sum(1 for _ in open(f'{resource_base}/numbers.data'))
addr_width = data_depth.bit_length()
print(f'data_width: {data_width}, data_depth: {data_depth}, addr_width: {addr_width}')

# MemUser module
class MemUser(Module):
    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width))}, 
            no_arbiter=True
        )
    @module.combinational
    def build(self, cycle_reg: RegArray, SM_reg: RegArray):
        width = self.rdata.dtype.bits
        radix_reg = RegArray(Int(width), 16, initializer=[0]*16)
        offset_reg = RegArray(Int(width), 1, initializer=[0])
        rdata = self.pop_all_ports(False)
        rdata = rdata.bitcast(Int(width))
        idx = (rdata >> offset_reg[0])[0:3]
        # Stage 1: Read Data into radix
        with Condition(SM_reg[0] == UInt(2)(1)):
            log("Phase 1: rdata: {:08x}", rdata)
            radix_reg[idx] = radix_reg[idx] + Int(width)(1)
        # Stage 3: Write Data to Memory's new address
        with Condition(SM_reg[0] != UInt(2)(1)):
            log("Phase 3: rdata: {:08x}", rdata)
        return radix_reg

# RadixReducer module
class RadixReducer(Module):
    def __init__(self, width):
        super().__init__(ports={})
    @module.combinational
    def build(self, radix_reg: RegArray, cycle_reg: RegArray):
        # Prefix sum
        with Condition(cycle_reg[0] < Int(data_width)(16)):
            radix_reg[cycle_reg[0]] = radix_reg[cycle_reg[0]] + radix_reg[cycle_reg[0]-Int(data_width)(1)]
            log("radix_reg[{}]: {:08x}", cycle_reg[0]-Int(data_width)(1), radix_reg[cycle_reg[0]-Int(data_width)(1)])
            cycle_reg[0] = cycle_reg[0] + Int(data_width)(1)
        return

# Driver module
class Driver(Module):
    def __init__(self):
        super().__init__(ports={},no_arbiter=True)

    @module.combinational
    def build(self, memory_user: Module, radix_reducer: Module, cycle_reg: RegArray, radix_reg: RegArray, SM_reg: RegArray):
        # Control Registers
        addr_reg = RegArray(UInt(addr_width), 1, initializer=[0])
        we = RegArray(Bits(1), 1, initializer=[0])
        wdata = RegArray(Bits(data_width), 1, initializer=[0])
        # wdata = (addr_reg[0] - UInt(addr_width)(1)).bitcast(Bits(data_width))

        # Build Memory
        read_cond = (addr_reg[0] < UInt(addr_width)(data_depth)) & (addr_reg[0] >= UInt(addr_width)(0))
        numbers_mem = SRAM(width=data_width, depth=data_depth, init_file=f'{resource_base}/numbers.data')
        numbers_mem.name = 'numbers_mem'
        numbers_mem.build(we=we[0], re=((SM_reg[0] == UInt(2)(1)) | (SM_reg[0] == UInt(2)(3))) & read_cond, wdata=wdata[0], addr=addr_reg[0], user=memory_user)
        # Stage Machine: 0 for start; 1 for read; 2 for sort
        with Condition(SM_reg[0] == UInt(2)(0)): # Stage 0: Stop
            pass
        with Condition(SM_reg[0] == UInt(2)(1)): # Stage 1: Read Data into radix
            with Condition(read_cond):
                numbers_mem.bound.async_called()    
                addr_reg[0] = addr_reg[0] + UInt(addr_width)(1)
            with Condition(~ read_cond):
                SM_reg[0] = UInt(2)(2)
                cycle_reg[0] = Int(data_width)(1)
                addr_reg[0] = addr_reg[0] - UInt(addr_width)(1)
        with Condition(SM_reg[0] == UInt(2)(2)): # Stage 2: Prefix sum the radix
            radix_reducer.async_called()
            with Condition(cycle_reg[0] == Int(data_width)(15)):
                SM_reg[0] = UInt(2)(3)
        with Condition(SM_reg[0] == UInt(2)(3)): # Stage 3: Write Data to Memory
            numbers_mem.bound.async_called()
            with Condition(addr_reg[0] > UInt(addr_width)(0)):
                addr_reg[0] = addr_reg[0] - UInt(addr_width)(1)
            with Condition(addr_reg[0] == UInt(addr_width)(0)):
                SM_reg[0] = UInt(2)(0)
        return

def build_system():
    sys = SysBuilder('radix_sort')
    with sys:
        SM_reg = RegArray(UInt(2), 1, initializer=[1])
        cycle_reg = RegArray(Int(data_width), 1, initializer=[0])
        # Create Radix Accumulator
        memory_user = MemUser(width=data_width)
        radix_reg = memory_user.build(cycle_reg=cycle_reg, SM_reg=SM_reg)
        # Create Radix Reducer
        radix_reducer = RadixReducer(width=data_width)
        radix_reducer.build(radix_reg, cycle_reg=cycle_reg)
        # Create driver
        driver = Driver()
        driver.build(memory_user, radix_reducer, cycle_reg=cycle_reg, radix_reg=radix_reg, SM_reg=SM_reg)

    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=2000,
        idle_threshold=2000,
        resource_base='',
        fifo_depth=1,
    )

    simulator_path, verilog_path = elaborate(sys, **conf)
    return sys, simulator_path, verilog_path

if __name__ == '__main__':
    sys, simulator_path, verilog_path = build_system()
    print("System built successfully!")
    utils.run_simulator(simulator_path)
    print("Simulation check completed!")