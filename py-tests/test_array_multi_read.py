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
            arr[0] = self.a


class Mod_b(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))

    @module.combinational
    def build(self, arr: Array):
        with Condition(~self.a[0: 0]):
            arr[0] = self.a

class Mod_c(Module):
    
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
        pass

    @module.combinational
    def build(self, mod_a: Mod_a, mod_b: Mod_b, mod_c: Mod_c):
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        new_v = v + Int(32)(1)
        cnt[0] = new_v
        mod_a.async_called(a = v)
        mod_b.async_called(a = v)
        mod_c.async_called(a = v)


def test_array_multi_read():

    sys = SysBuilder('array_multi_read')
    with sys:
        arr = RegArray(Int(32), 1)
        
        mod_a = Mod_a()
        mod_a.build(arr)

        mod_b = Mod_b()
        mod_b.build(arr)

        mod_c = Mod_c()
        mod_c.build(arr)

        driver = Driver()
        driver.build(mod_a, mod_b, mod_c)

    print(sys)

    simulator_path = elaborate(sys)
    raw = utils.run_simulator(simulator_path)

    print(raw)
        
        

if __name__ == '__main__':
    test_array_multi_read()