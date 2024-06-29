import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Mod_a(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))

    @module.combinational
    def build(self, arr: Array):
        with Condition(self.a[0: 0]):
            with Condition(self.a[1: 1]):
                pass
            arr[0] = self.a
        with Condition(~self.a[0: 0]):
            arr[0] = self.a + Int(32)(1)


class Driver(Module):
    
    @module.constructor
    def __init__(self):
        pass
    
    @module.combinational
    def build(self, mod_a: Mod_a):
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        new_v = v + Int(32)(1)
        cnt[0] = new_v
        mod_a.async_called(a = v)


def test_array_multi_write_in_same_module():
    sys =  SysBuilder('array_multi_write_in_same_module')
    with sys:

        arr = RegArray(Int(32), 1)

        mod_a = Mod_a()
        mod_a.build(arr)

        driver = Driver()
        driver.build(mod_a)

    print(sys)

    simulator_path = elaborate(sys)
    raw = utils.run_simulator(simulator_path)

    print(raw)

if __name__ == '__main__':
    test_array_multi_write_in_same_module()