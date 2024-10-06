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
        with Condition(a[0: 0]):
            arr[0] = a


class ModB(Module):
    
    def __init__(self):
        ports = {
            'a': Port(Int(32))
        }
        super().__init__(ports=ports)

    @module.combinational
    def build(self, arr: Array):
        a = self.pop_all_ports(True)
        with Condition(~a[0: 0]):
            arr[0] = a

class ModC(Module):
    
    def __init__(self):
        ports = {
            'a': Port(Int(32))
        }
        super().__init__(ports=ports)

    @module.combinational
    def build(self, arr: Array):
        a = self.pop_all_ports(True)
        v = arr[0]
        log("a = {} arr = {}", a, v)
    
class Driver(Module):
     
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, mod_a: ModA, mod_b: ModB, mod_c: ModC):
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        new_v = v + Int(32)(1)
        cnt[0] = new_v
        mod_a.async_called(a = v)
        mod_b.async_called(a = v)
        mod_c.async_called(a = v)


def check(raw):
    for i in raw.split('\n'):
        if 'a =' in i and 'arr =' in i:
            line_toks = i.split()
            a = line_toks[-4]
            arr = line_toks[-1]
            if a == '0':
                assert int(arr) == 0
                continue
            assert int(arr) == (int(a) - 1)

def test_array_multi_read():

    sys = SysBuilder('array_multi_read')
    with sys:
        arr = RegArray(Int(32), 1)
        
        mod_a = ModA()
        mod_a.build(arr)

        mod_b = ModB()
        mod_b.build(arr)

        mod_c = ModC()
        mod_c.build(arr)

        driver = Driver()
        driver.build(mod_a, mod_b, mod_c)

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())
    
    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_array_multi_read()
