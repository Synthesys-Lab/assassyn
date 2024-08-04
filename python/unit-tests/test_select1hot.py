import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.cond = Port(Int(5))
        
    @module.combinational
    def build(self):
        cond = RegArray(Int(5), 1, initializer = [0b00001])
        values = RegArray(Int(32), 5, initializer = [1, 2, 4, 8, 16])

        gt = self.cond
        mux = gt.select1hot(values[0], values[1], values[2], values[3], values[4])

        log("onehot select {:b}: {}", cond, mux)

class Testbench(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, driver: Module):
        for i in range(1, 201):
            with Cycle(i):
                cond = 1 << (i % 5)
                driver.async_called(cond = Int(5)(cond))


def check(raw: str):
    for i in raw.splitlines():
        if 'onehot select' in i:
            a = i.split()[-3]
            b = i.split()[-1]
            assert int(a, 2) == int(b)

def test_select1hot():
    sys = SysBuilder('select1hot')
    with sys:
        driver = Driver()
        driver.build()

        testbench = Testbench()
        testbench.build(driver)
        

    print(sys)
    # simulator_path, verilator_path = elaborate(sys, verilog='verilator')

    # raw = utils.run_simulator(simulator_path)
    # check(raw)

if __name__ == '__main__':
    test_select1hot()
