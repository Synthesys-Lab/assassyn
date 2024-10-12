import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Driver(Module):

    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self):
        cnt = RegArray(UInt(32), 1, initializer=[10])
        log('cnt: {}', cnt[0]);

def check(raw):
    for i in raw.split('\n'):
        if 'cnt:' in i:
            assert int(i.split()[-1]) == 10

def test_driver():
    sys = SysBuilder('reg_init')
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
    test_driver()
