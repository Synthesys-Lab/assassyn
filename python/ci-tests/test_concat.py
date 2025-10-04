import pytest

from assassyn.frontend import *
from assassyn.test import run_test
from assassyn import utils


class Adder(Module):

    def __init__(self):
        ports={
            'msb': Port(Int(32)),
            'lsb': Port(Int(32))
        }
        super().__init__(
            ports=ports ,
        )

    @module.combinational
    def build(self):
        msb, lsb = self.pop_all_ports(True)
        c = concat(msb, lsb)
        log("concat: {} << 32 + {} = {}", msb, lsb, c)


class Driver(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, add: Adder):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        add.async_called(msb = cnt[0], lsb = cnt[0])

def check_concat(raw):
    cnt = 0
    for i in raw.split('\n'):
        if f'concat:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-7]
            cnt += 1
            assert (int(a) << 32) + int(b) == int(c)
    assert cnt == 200 - 1, f'cnt:{cnt} != 200 - 1'

def test_concat():
    def top():
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    run_test(
        'concat',
        top=top,
        checker=check_concat,
        sim_threshold=200,
        idle_threshold=200
    )


if __name__ == '__main__':
    test_concat()
