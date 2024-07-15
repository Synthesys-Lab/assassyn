import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class ModA(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))

    @module.combinational
    def build(self, arr: Array):
        v = self.a[0: 0]
        with Condition(v):
            arr[0] = self.a
        with Condition(~v):
            arr[0] = self.a + Int(32)(1)

class ModC(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))

    @module.combinational
    def build(self, arr: Array):
        v = arr[0]
        log("a = {} arr = {}", self.a, v)

class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
    
    @module.combinational
    def build(self, mod_a: ModA, mod_c: ModC):
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        new_v = v + Int(32)(1)
        cnt[0] = new_v
        mod_a.async_called(a = v)
        mod_c.async_called(a = v)


def test_array_multi_write_in_same_module():
    sys =  SysBuilder('array_multi_write_in_same_module')
    with sys:

        arr = RegArray(Int(32), 1)

        mod_a = ModA()
        mod_a.build(arr)

        mod_c = ModC()
        mod_c.build(arr)

        driver = Driver()
        driver.build(mod_a, mod_c)

    print(sys)

    simulator_path = elaborate(sys)
    raw = utils.run_simulator(simulator_path)

    print(raw)

if __name__ == '__main__':
    test_array_multi_write_in_same_module()
