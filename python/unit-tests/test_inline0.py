import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


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
        cnt[0] = v
        adder(v, v)

def check(raw):
    for i in raw.split('\n'):
        if f'adder:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)

def test_inline0():
    sys = SysBuilder('inline0')
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
    test_inline0()
