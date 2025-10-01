import pytest

from assassyn.frontend import *
from assassyn.test import run_test


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
    def top():
        driver = Driver()
        driver.build()

    run_test('reg_init', top, check)


if __name__ == '__main__':
    test_driver()
