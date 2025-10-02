import pytest

from assassyn.frontend import *
from assassyn.test import run_test


class ModA(Module):

    def __init__(self):
        super().__init__(ports={'a': Port(Int(32))})


    @module.combinational
    def build(self, arr: Array):
        a = self.pop_all_ports(True)
        v = a[0: 0]
        with Condition(v):
            (arr&self)[0] <= a
        with Condition(~v):
            (arr&self)[0] <= a + Int(32)(1)

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
        (cnt & self)[0] <= new_v
        mod_a.async_called(a = v)
        mod_c.async_called(a = v)


def test_array_multi_write():
    def top():
        arr = RegArray(Int(32), 1)

        mod_a = ModA()
        mod_a.build(arr)

        mod_c = ModC()
        mod_c.build(arr)

        driver = Driver()
        driver.build(mod_a, mod_c)

    def checker(output):
        # Basic check that the test ran without errors
        assert output is not None

    run_test('array_multi_write', top, checker)

if __name__ == '__main__':
    test_array_multi_write()
