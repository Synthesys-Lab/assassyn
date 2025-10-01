import pytest

from assassyn.frontend import *
from assassyn.test import run_test

class Driver(Module):

    def __init__(self):
            super().__init__(ports={},no_arbiter=True,)


    @module.combinational
    def build(self):
        cond = RegArray(Int(5), 1, initializer=[0])
        values = RegArray(Int(32), 5, initializer = [1, 2, 4, 8, 16])

        gt = Int(5)(1) << cond[0]
        mux = gt.select1hot(values[0], values[1], values[2], values[3], values[4])

        log("onehot select 0b{:b} from [1,2,4,8,16]: {}", gt, mux)
        (cond & self)[0] <= (cond[0] + Int(5)(1)) % Int(5)(5)

def top():
    driver = Driver()
    driver.build()

def check(raw: str):
    for i in raw.splitlines():
        if 'onehot select' in i:
            a = i.split()[-4]
            b = i.split()[-1]
            assert int(a, 2) == int(b)

def test_select1hot():
    run_test('select1hot', top, check)

if __name__ == '__main__':
    test_select1hot()
