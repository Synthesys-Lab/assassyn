import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Driver(Module):

    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self):
        cnt = RegArray(UInt(32), 1, initializer=[10])
        log('cnt: {}', cnt[0]);

def check(raw):
    for i in raw.split('\n'):
        if 'cnt:' in i:
            assert int(i.split()[-1]) == 10

def test_driver():
    sys = SysBuilder('driver')
    with sys:
        driver = Driver()
        driver.build()

    simulator_path, verilator_path = elaborate(sys, verilog=utils.verilator_path())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_driver()
