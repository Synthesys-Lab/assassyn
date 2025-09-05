import pytest

from assassyn.backend import elaborate
from assassyn.frontend import *
from assassyn import utils

class Sub(Module):
    def __init__(self):
        ports={
            'a': Port(Int(32)),
            'b': Port(Int(32))
        }
        super().__init__(
            ports=ports,
        )

    @module.combinational
    def build(self):
        a, b = self.pop_all_ports(True)
        c = a - b
        log("Subtractor: {} - {} = {}", a, b, c)

class Lhs(Module):


    def __init__(self):
        super().__init__(
            ports={'a': Port(Int(32))},
        )

    @module.combinational
    def build(self, sub: Sub):
        a = self.pop_all_ports(True)
        bound = sub.bind(a = a)
        return bound

class Driver(Module):
    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self, lhs: Lhs, rhs):
        cnt = RegArray(Int(32), 1)
        v = cnt[0] + Int(32)(1)
        (cnt & self)[0] <= v

        lhs.async_called(a = v + v)
        rhs.async_called(b = v)


def check(raw):
    cnt = 0
    for i in raw.split('\n'):
        if f'Subtractor:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(b) - int(a) == int(c)
            cnt += 1
    assert cnt == 100 - 2, f'cnt: {cnt} != 100'

def test_imbalance():
    sys =  SysBuilder('imbalance')
    with sys:
        sub = Sub()
        sub.build()

        lhs = Lhs()
        rhs = lhs.build(sub)

        driver = Driver()
        driver.build(lhs, rhs)


    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_imbalance()
