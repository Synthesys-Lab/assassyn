import pytest

from assassyn import utils
from assassyn.test import run_test
from assassyn.frontend import *

class Driver(Module):

    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self):
        a = [RegArray(Int(32), 1) for _ in range(4)]
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        cnt[0] = v + Int(32)(1)
        (a[0] & self)[0] <= v
        (a[1] & self)[0] <= v
        (a[2] & self)[0] <= v
        (a[3] & self)[0] <= v
        a_sum = a[0][0] + a[1][0] + a[2][0] + a[3][0]
        log("sum(a[:]) = {}", a_sum)

def top():
    driver = Driver()
    driver.build()

def checker(raw):
    parser = utils.parse_simulator_cycle if 'cycle' in raw.lower() else utils.parse_verilator_cycle
    for i in raw.split('\n'):
        if 'sum(a[:])' in i:
            line_toks = i.split()
            cycle = parser(line_toks)
            assert (int(line_toks[-1]) % 4) == 0
            assert max(cycle - 2, 0) * 4 == int(line_toks[-1])

def test_array_partition0():
    run_test('array_partition0', top, checker)


if __name__ == '__main__':
    test_array_partition0()
