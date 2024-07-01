import pytest

from assassyn.backend import elaborate
from assassyn.frontend import *
from assassyn import utils

class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
    
    @module.combinational
    def build(self):
        log("Hello, World!")


def test_helloworld():
    
    sys = SysBuilder('helloworld')

    with sys:
        driver = Driver()
        driver.build()

    print(sys)

    simulator_path = elaborate(sys)
    
    raw = utils.run_simulator(simulator_path)

    print(raw)

    for i in raw.split('\n'):
        if f'[{driver.synthesis_name().lower()}]' in i:
            assert "Hello, World!" in i

if __name__ == '__main__':
    test_helloworld()
