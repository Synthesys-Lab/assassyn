import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Driver(Module):

    @module.constructor
    def __init__(self):
        pass

    @module.combinational
    def build(self):
        a = RegArray(Int(256), 1, initializer=[0])
        b = RegArray(Int(256), 1, initializer=[1])

        aa = a[0]
        bb = b[0]
        cc = aa + bb

        log('fib: {} + {} = {}', aa, bb, cc)

        a[0] = bb
        b[0] = cc


def test_fib():

    sys = SysBuilder('fib')
    with sys:
        driver = Driver()
        driver.build()

    simulator_path = elaborate(sys)

    raw = utils.run_simulator(simulator_path)

    print(raw)

    expect_a = 0
    expect_b = 1
    for i in raw.split('\n'):
        if f'[{driver.as_operand().lower()}]' in i:
            line_toks = i.split()
            c = line_toks[-1]
            b = line_toks[-3]
            a = line_toks[-5]
            assert int(a) == expect_a
            assert int(b) == expect_b
            expect_a = int(b)
            expect_b = int(c)
            assert int(a) + int(b) == int(c)


if __name__ == '__main__':
    test_fib()
