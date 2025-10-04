import pytest

from assassyn.frontend import *
from assassyn.test import run_test


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        i32 = Int(32)(1)
        b32 = i32.bitcast(Bits(32))
        i64z = i32.zext(Int(64))
        i64s = i32.sext(Int(64))

        b32_array = RegArray(Bits(32), 1)
        (b32_array & self)[0] <= b32
        i64z_array = RegArray(Int(64), 1)
        i64s_array = RegArray(Int(64), 1)
        (i64z_array&self)[0] <= i64z
        (i64s_array&self)[0] <= i64s
        log("Cast: {} {} {}", b32_array[0], i64z_array[0], i64s_array[0]);


def top():
    driver = Driver()
    driver.build()


def checker(raw: str):
    cnt = 0
    for i in raw.splitlines():
        cnt += 'Cast:' in i
    assert cnt == 100


def test_dt_conv():
    run_test('dt_conv', top, checker)


if __name__ == '__main__':
    test_dt_conv()
