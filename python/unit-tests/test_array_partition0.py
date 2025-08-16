import pytest

from assassyn import utils
from assassyn.backend import elaborate
from assassyn.frontend import *

class Driver(Module):
      
    def __init__(self):
            super().__init__(ports={})
            
    @module.combinational
    def build(self):
        a = RegArray(Int(32), 4, partition='full')
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        cnt[0] = v + Int(32)(1)
        a[0] = v
        a[1] = v
        a[2] = v
        a[3] = v
        a_sum = a[0] + a[1] + a[2] + a[3]
        log("sum(a[:]) = {}", a_sum)

def check(raw, parser):
    for i in raw.split('\n'):
        if 'sum(a[:])' in i:
            line_toks = i.split()
            cycle = parser(line_toks)
            assert (int(line_toks[-1]) % 4) == 0
            assert max(cycle - 2, 0) * 4 == int(line_toks[-1])

def test_array_partition0():
    sys = SysBuilder('array_partition0')
    with sys:
        driver = Driver()
        driver.build()

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw, utils.parse_simulator_cycle)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw, utils.parse_verilator_cycle)


if __name__ == '__main__':
    test_array_partition0()
