import pytest

from assassyn.frontend import *
from assassyn.test import run_test

class Testbench(Module):

    __test__ = False

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
    def top():
        testbench = Testbench()
        testbench.build()

    run_test('testbench', top, check, verilog=True)


if __name__ == '__main__':
    test_testbench()
