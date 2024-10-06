import pytest

from assassyn.backend import elaborate
from assassyn.frontend import *
from assassyn import utils

class Driver(Module):
    
    def __init__(self): 
        super().__init__(
            ports={} ,
        )  
        
    @module.combinational
    def build(self):
        log("Hello, World!")


def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        cnt += "Hello, World!" in i
    assert cnt == 100, "Hello, World! not found in raw output"


def test_helloworld():
    
    sys = SysBuilder('helloworld')

    with sys:
        driver = Driver()
        driver.build()

    simulator_path, verilog_path = elaborate(sys, verilog=utils.has_verilator())
    
    raw = utils.run_simulator(simulator_path)
    check_raw(raw)

    if verilog_path:
        raw = utils.run_verilator(verilog_path)
        check_raw(raw)


if __name__ == '__main__':
    test_helloworld()
