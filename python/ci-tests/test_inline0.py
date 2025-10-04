import pytest

from assassyn.frontend import *
from assassyn.test import run_test


def adder(a, b):
    c = a + b
    log("adder: {} + {} = {}", a, b, c)

class Driver(Module):

    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self):
        cnt = RegArray(Int(32), 1)
        k = cnt[0]
        v = k + Int(32)(1)
        (cnt & self)[0] <= v
        adder(v, v)

def top():
    driver = Driver()
    driver.build()

def check(raw):
    for i in raw.split('\n'):
        if f'adder:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)

def test_inline0():
    run_test('inline0', top, check)


if __name__ == '__main__':
    test_inline0()
