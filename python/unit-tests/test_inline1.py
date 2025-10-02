import pytest

from assassyn.frontend import *
from assassyn.test import run_test

def ae(a, b):
    c = a + b
    eq = (a == b)
    return (c, eq)

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        cnt = RegArray(Int(32), 1)
        k = cnt[0]
        v = k + Int(32)(1)
        (cnt & self)[0] <= v
        a, e = ae(v, v)
        log("add: {} + {} = {}", v, v, a)
        log("eq: {} == {} ? {}", v, v, e)

def check(raw):
    for i in raw.split('\n'):
        line_toks = i.split()
        if 'add:' in i:
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
        elif 'eq:' in i:
            a = line_toks[-5]
            b = line_toks[-3]
            c = line_toks[-1]
            assert bool(c) == (a == b)

def top():
    driver = Driver()
    driver.build()

def test_inline1():
    run_test('inline1', top, check)

if __name__ == '__main__':
    test_inline1()
