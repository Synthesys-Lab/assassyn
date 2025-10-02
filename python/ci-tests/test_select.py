import pytest

from assassyn.frontend import *
from assassyn.test import run_test

class Driver(Module):

    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self):
        rng0 = RegArray(UInt(32), 1)
        rng1 = RegArray(UInt(32), 1)

        v0 = rng0[0]
        v1 = rng1[0]

        v0 = v0 * UInt(32)(12345)
        v1 = v1 * UInt(32)(54321)

        rand0 = v0 + UInt(64)(67890)
        rand1 = v1 + UInt(64)(9876)

        rand0 = rand0[0: 31].bitcast(UInt(32))
        rand1 = rand1[0: 31].bitcast(UInt(32))

        (rng0 & self)[0] <= rand0
        (rng1 & self)[0] <= rand1

        gt = rand0 > rand1
        mux = gt.select(rand0, rand1)

        log("select: {} >? {} = {}", rand0, rand1, mux)


def check(raw: str):
    for i in raw.splitlines():
        if 'select:' in i:
            a = i.split()[-5]
            b = i.split()[-3]
            c = i.split()[-1]
            assert max(int(a), int(b)) == int(c)


def top():
    driver = Driver()
    driver.build()


def test_select():
    run_test('select', top, check)


if __name__ == '__main__':
    test_select()
