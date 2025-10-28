import pytest

from assassyn.frontend import *
from assassyn.test import run_test
from assassyn import utils

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        initializer_a = [[1],[2],[3],[4]]
        a = [RegArray(Int(32), 1,initializer=initializer_a[i]) for i in range(4)]
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        new_v = v + Int(32)(1)
        (cnt & self)[0] <= new_v
        idx0 = v[0: 1]
        idx1 = new_v[0: 1]
        for i in range(4):
            with Condition(idx0 == UInt(2)(i)):
                (a[i] & self)[0] <= ((v * v)[0:31].bitcast(Int(32)))
            with Condition(idx1 == UInt(2)(i)):
                (a[i] & self)[0] <= new_v + new_v
        for i in range(4):
            for j in range(4):
                with Condition(idx0 == UInt(2)(i)) and Condition(idx1 == UInt(2)(j)):
                    a_sum = a[i][0] + a[j][0]
                    log("a[idx0][0] + a[idx1][0] = {}", a_sum)


def top():
    driver = Driver()
    driver.build()


def checker(raw):
    print(raw)
    a = [1, 2, 3, 4]
    # Determine which parse function to use based on output format
    if "cycle:" in raw.lower():
        parse_cycle = utils.parse_verilator_cycle
    else:
        parse_cycle = utils.parse_simulator_cycle

    for i in raw.split('\n'):
        if "a[idx0] + a[idx1]" in i:
            line_toks = i.split()
            cycle = parse_cycle(line_toks) - 1
            idx0 = cycle % 4
            idx1 = (cycle + 1) % 4
            expect = a[idx0] + a[idx1]
            assert expect == int(line_toks[-1]), f"@cycle: {cycle}: expect {expect}, got {line_toks[-1]}"
            a[idx0] = cycle * cycle
            a[idx1] = (cycle + 1) * 2


def test_array_partition1():
    run_test('array_partition1', top, checker, verilog=True)


if __name__ == '__main__':
    test_array_partition1()
