# There should be a driver module to async call the radix sort module
# In the radix sort module, it should utilize a memory module.
# The user of the memory is a memUser module, which will compare two numbers and write the smaller one to the memory.
import os
import math

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

# Resource base path
current_path = os.path.dirname(os.path.abspath(__file__))
resource_base = f'{current_path}/workload/'

# Data width and length
data_width = 32
# read numbers.data file
data_len = sum(1 for _ in open(f'{resource_base}/numbers.data'))
addr_width = math.ceil(math.log2(data_len))

# MemUser module
class MemUser(Module):
    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width))}, 
        )
    @module.combinational
    def build(self):
        width = self.rdata.dtype.bits
        rdata = self.pop_all_ports(False)
        rdata = rdata.bitcast(Int(width))
        log("rdata: {:08x}", rdata)
        # radix = RegArray(Bits(width), 16)
        # idx = (rdata >> x)[0:3]
        # radix[idx] = radix[idx] + Int(width)(1)
        # log("rdata_reg: {:08x}", rdata_reg[0])
        return

class Driver(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, mem_user: Module):
        # Control registers
        read_reg = RegArray(Int(1), 1, initializer=[1])
        addr_reg = RegArray(Int(data_width), 1, initializer=[0])
        # Memory
        numbers_mem = SRAM(width=data_width, depth=data_len, init_file=f'{resource_base}/numbers.data')
        numbers_mem.name = 'numbers_mem'
        cond = addr_reg[0] < Int(data_width)(data_len)
        numbers_mem.build(we=Bits(1)(0),re=cond,wdata=Bits(data_width)(0),addr=addr_reg[0].bitcast(Bits(addr_width)),user=mem_user)
        with Condition(cond):
            numbers_mem.bound.async_called()
            addr_reg[0] = addr_reg[0] + Int(data_width)(1)

def build_system():
    sys = SysBuilder('radix_sort')
    
    with sys:
        # Create memory user
        mem_user = MemUser(32)
        mem_user.build()
        # Create driver
        driver = Driver()
        driver.build(mem_user)

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