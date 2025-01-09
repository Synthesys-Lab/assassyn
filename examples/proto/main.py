import pytest

import assassyn
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Testbench(Module):    
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        with Cycle(1):
            log("OK")

def test_proto():
    sys = SysBuilder('protobuf')
    with sys:        
        testbench = Testbench()
        testbench.build()

    config = assassyn.backend.config(sim_threshold=20, idle_threshold=20, verilog=utils.has_verilator())
    simulator_path, verilator_path = elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    print(raw)
    
    if verilator_path:
        raw = utils.run_verilator(verilator_path)


if __name__ == '__main__':
    test_proto()
