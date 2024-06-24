import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Driver(Module):

    @module.constructor
    def __init__(self):
        pass

    @module.combinational
    def build(self):
        cnt = RegArray(UInt(32), 1, 1234)
        log('cnt: {}', cnt[0])


def test_driver():
    sys = SysBuilder('driver')
    with sys:
        driver = Driver()
        driver.build()

    print(sys)
    simulator_path = elaborate(sys)

    raw = utils.run_simulator(simulator_path)

    print(raw)


if __name__ == '__main__':
    test_driver()
