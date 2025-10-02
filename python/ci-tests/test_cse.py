import pytest

from assassyn.frontend import *
from assassyn.test import run_test


class Adder(Module):

    def __init__(self):
        ports={
            'a': Port(Int(32)),
            'b': Port(Int(32))
        }
        super().__init__(
            ports=ports ,
        )

    @module.combinational
    def build(self):
        a, b = self.pop_all_ports(True)
        c = a + b
        log("Adder: {} + {} = {}", a, b, c)


class Driver(Module):
    def __init__(self):
        super().__init__(
            ports={} ,
        )

    @module.combinational
    def build(self, add: Adder):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        cond = cnt[0] < Int(32)(100)
        with Condition(cond):
            add.async_called(a = cnt[0], b = cnt[0])
        with Condition(cond):
            log("Done")

def check(raw):
    cnt = 0
    for i in raw.split('\n'):
        if ('Adder:' in i) and ('@line:' in i):
            print(i)
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 100, f'cnt: {cnt} != 100 - 1'


def top():
    adder = Adder()
    adder.build()

    driver = Driver()
    driver.build(adder)


def test_cond_cse():
    run_test('cond_cse', top, check, sim_threshold=200, idle_threshold=200)


if __name__ == '__main__':
    test_cond_cse()
