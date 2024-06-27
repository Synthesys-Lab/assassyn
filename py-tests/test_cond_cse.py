import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Adder(Module):
    @module.constructor
    def __init__(self):
        super().__init__()
        self.add_a = Port(Int(32))
        self.add_b = Port(Int(32))
    
    @module.combinational
    def build(self):
        c = self.add_a + self.add_b
        log("Adder: {} + {} = {}", self.add_a, self.add_b, c)


class Driver(Module):
    @module.constructor
    def __init__(self):
        pass
    
    @module.combinational
    def build(self, add: Adder):
        cnt = RegArray(Int(32), 1)
        cnt[0] = cnt[0] + Int(32)(1)
        cond = cnt[0] < Int(32)(100)
        with Condition(cond):
            add.async_called(add_a = cnt[0], add_b = cnt[0])
        with Condition(cond):
            log("Done")

def test_cond_cse():
    sys = SysBuilder('cond_cse')
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
    test_cond_cse()
