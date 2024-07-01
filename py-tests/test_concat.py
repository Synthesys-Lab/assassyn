import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Adder(Module):
    @module.constructor
    def __init__(self):
        super().__init__()
        self.msb = Port(Int(32))
        self.lsb = Port(Int(32))
    
    @module.combinational
    def build(self):
        c = self.msb.concat(self.lsb)
        log("add with pred: {} << 32 + {} = {}", self.msb, self.lsb, c)


class Driver(Module):
    @module.constructor
    def __init__(self):
        super().__init__()
    
    @module.combinational
    def build(self, add: Adder):
        cnt = RegArray(Int(32), 1)
        cnt[0] = cnt[0] + Int(32)(1)
        add.async_called(msb = cnt[0], lsb = cnt[0])

def test_concat():
    sys = SysBuilder('concat')
    with sys:
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    print(sys)

    simulator_path = elaborate(sys, sim_threshold=200, idle_threshold=200)

    raw = utils.run_simulator(simulator_path)

    cnt = 0
    for i in raw.split('\n'):
        if f'[{adder.synthesis_name().lower()}]' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-7]
            cnt += 1
            assert (int(a) << 32) + int(b) == int(c)
    assert cnt == 200, f'cnt:{cnt} != 200'

if __name__ == '__main__':
    test_concat()
