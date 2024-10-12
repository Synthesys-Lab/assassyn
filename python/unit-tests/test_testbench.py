import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Testbench(Module):

    def __init__(self):
            super().__init__(ports={})
        

    @module.combinational
    def build(self):
        with Cycle(1):
            log('tbcycle 1')
        with Cycle(2):
            log('tbcycle 2')
        with Cycle(82):
            log('tbcycle 82')

def check(raw):
    expected = 0
    expects = [1, 2, 82]
    for i in raw.split('\n'):
        if 'tbcycle' in i:
            assert f'tbcycle {expects[expected]}' in i, f'tbcycle {expects[expected]} NOT IN {i}'
            expected += 1
    assert expected == 3, f'{expected} != 3'

def test_testbench():
    sys = SysBuilder('testbench')
    with sys:
        testbench = Testbench()
        testbench.build()

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_testbench()
