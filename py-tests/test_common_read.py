import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Adder(Module):
    
    @module.constructor
    def __init__(self):
        self.a = Port(Int(32))
        self.b = Port(Int(32))
        
    @module.combinational
    def build(self):
        c = self.a + self.b
        log("adder: {} + {} = {}", self.a, self.b, c)


class Driver(Module):
    
    @module.constructor
    def __init__(self):
        pass

    @module.combinational
    def build(self, adder: Adder):
        cnt = RegArray(Int(32), 1)
        new_cnt = cnt[0] + Int(32)(1)
        cnt[0] = new_cnt
        cond = cnt[0] < Int(32)(100)
        with Condition(cond):
            adder.async_called(a = cnt[0], b = cnt[0])


def test_common_read():
    sys = SysBuilder('common_read')
    with sys:
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    simulator_path = elaborate(sys, sim_threshold=200, idle_threshold=200)

    raw = utils.run_simulator(simulator_path)

    cnt = 0
    for i in raw.split('\n'):
        if f'[{adder.synthesis_name().lower()}]' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 100, f'cnt: {cnt} != 100'


if __name__ == '__main__':
    test_common_read()
