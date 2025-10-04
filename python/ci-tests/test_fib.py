import pytest

from assassyn.frontend import *
from assassyn.test import run_test

class Driver(Module):

    def __init__(self):
        super().__init__(
            ports={} ,
        )

    @module.combinational
    def build(self):
        a = RegArray(Int(256), 1, initializer=[0])
        b = RegArray(Int(256), 1, initializer=[1])

        aa = a[0]
        bb = b[0]
        cc = aa + bb

        log('fib: {} + {} = {}', aa, bb, cc)

        (a & self)[0] <= bb
        b[0] = cc

def top():
    driver = Driver()
    driver.build()

def check(raw):
    expect_a = 0
    expect_b = 1
    for i in raw.split('\n'):
        if f'fib:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            b = line_toks[-3]
            a = line_toks[-5]
            assert int(a) == expect_a
            assert int(b) == expect_b
            expect_a = int(b)
            expect_b = int(c)
            assert int(a) + int(b) == int(c)

def test_fib():
    run_test('fib', top, check)


if __name__ == '__main__':
    test_fib()
