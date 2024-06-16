import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

@pytest.mark.skip(reason='Testbench is intentionally skipped!')
class Testbench(Module):

    @module.constructor
    def __init__(self):
        pass

    @module.combinational
    def build(self):
        with Cycle(0):
            log('cycle 0')
        with Cycle(2):
            log('cycle 2')
        with Cycle(82):
            log('cycle 82')

def test_testbench():
    sys = SysBuilder('testbench')
    with sys:
        testbench = Testbench()
        testbench.build()

    print(sys)

    simulator_path = elaborate(sys)

    raw = utils.run_simulator(simulator_path)

    expected = 0
    expects = [0, 2, 82]
    for i in raw.split('\n'):
        if '[testbench]' in i:
            assert f'cycle {expects[expected]}' in i
            expected += 1
    assert expected == 3, f'{expected} != 3'

if __name__ == '__main__':
    test_testbench()
