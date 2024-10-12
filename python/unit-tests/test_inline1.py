import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

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
        cnt[0] = v
        a, e = ae(v, v)
        log("add: {} + {} = {}", v, v, a)
        log("eq: {} == {} ? {}", v, v, e)

def check(raw):
    for i in raw.split('\n'):
        line_toks = i.split()
        if 'add' in i:
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
        elif 'eq' in i:
            a = line_toks[-5]
            b = line_toks[-3]
            c = line_toks[-1]
            assert bool(c) == (a == b)

def test_inline1():
    sys = SysBuilder('inline1')
    with sys:
        driver = Driver()
        driver.build()

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_inline1()
