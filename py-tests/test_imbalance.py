import pytest

from assassyn.backend import elaborate
from assassyn.frontend import *
from assassyn import utils
from assassyn.expr import Bind

class Sub(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.sub_a = Port(Int(32))
        self.sub_b = Port(Int(32))

    @module.combinational
    def build(self):
        c = self.sub_a - self.sub_b
        log("Subtractor: {} - {} = {}", self.sub_a, self.sub_b, c)

class Lhs(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.lhs_a = Port(Int(32))

    @module.combinational
    def build(self, sub: Sub):
        bound = sub.bind(sub_a = self.lhs_a)
        return bound

class Driver(Module):

    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, lhs: Lhs, rhs: Bind):
        cnt = RegArray(Int(32), 1)
        v = cnt[0] + Int(32)(1)
        cnt[0] = v

        lhs.async_called(lhs_a = v + v)
        rhs.async_called(sub_b = v) 


def test_imbalance():
    sys =  SysBuilder('imbalance')
    with sys:
        sub = Sub()
        sub.build()

        lhs = Lhs()
        rhs = lhs.build(sub)

        driver = Driver()
        driver.build(lhs, rhs)

    print(sys)

    simulator_path = elaborate(sys, verilog='verilator')

    raw = utils.run_simulator(simulator_path)

    print(raw)

    cnt = 0
    for i in raw.split('\n'):
        if f'[{sub.synthesis_name().lower()}]' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(b) - int(a) == int(c)
            cnt += 1
    assert cnt == 100 - 1, f'cnt: {cnt} != 100'

    
if __name__ == '__main__':
    test_imbalance()
