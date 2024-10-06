import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class ModA(Module):

    def __init__(self):
        super().__init__(ports={'a': Port(Int(32))})


    @module.combinational
    def build(self, arr: Array):
        a = self.pop_all_ports(True)
        v = a[0: 0]
        with Condition(v):
            arr[0] = a
        with Condition(~v):
            arr[0] = a + Int(32)(1)

class ModC(Module):

    def __init__(self):
        super().__init__(ports={'a': Port(Int(32))})

    @module.combinational
    def build(self, arr: Array):
        a = self.pop_all_ports(True)
        v = arr[0]
        log("a = {} arr = {}", a, v)

class Driver(Module):
    
    def __init__(self):
        super().__init__(ports={})
    
    @module.combinational
    def build(self, mod_a: ModA, mod_c: ModC):
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        new_v = v + Int(32)(1)
        cnt[0] = new_v
        mod_a.async_called(a = v)
        mod_c.async_called(a = v)


def test_array_multi_write():
    sys =  SysBuilder('array_multi_write')
    with sys:

        arr = RegArray(Int(32), 1)

        mod_a = ModA()
        mod_a.build(arr)

        mod_c = ModC()
        mod_c.build(arr)

        driver = Driver()
        driver.build(mod_a, mod_c)

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())
    
    utils.run_simulator(simulator_path)
    
    if verilator_path:
        utils.run_verilator(verilator_path)

if __name__ == '__main__':
    test_array_multi_write()
