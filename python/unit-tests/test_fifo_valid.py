import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.expr import Bind

class Sub(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.sub_a = Port(Int(32))
        self.sub_b = Port(Int(32))

    @module.wait_until
    def wait_until(self):
        a_valid = self.sub_a.valid()
        b_valid = self.sub_b.valid()
        both_valid = a_valid & b_valid
        return both_valid

    @module.combinational
    def build(self):
        c = self.sub_a - self.sub_b
        log("sub: {} - {} = {}", self.sub_a, self.sub_b, c);


class Lhs(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.v = Port(Int(32))

    @module.combinational
    def build(self, sub: Sub):
        rhs = sub.bind(sub_a = self.v)
        return rhs

class Driver(Module):
    
        @module.constructor
        def __init__(self):
            super().__init__()

        @module.combinational
        def build(self, rhs: Bind, lhs: Lhs):
            cnt = RegArray(Int(32), 1)
            k = cnt[0]
            v = k + Int(32)(1)
            cnt[0] = v
            vv = v + v
            lhs.async_called(v = vv)
            rhs.async_called(sub_b = v)
 

def check(raw):
    cnt = 0
    for i in raw.split('\n'):
        if f'sub:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(b) - int(a) == int(c)
            cnt += 1
    assert cnt == 100 - 2, f'cnt: {cnt} != 100 - 2'


def test_fifo_valid():
    sys = SysBuilder('fifo_valid')
    with sys:
        sub = Sub()
        sub.wait_until()
        sub.build()
        
        lhs = Lhs()
        rhs = lhs.build(sub)

        driver = Driver()
        driver.build(rhs, lhs)

    print(sys)

    simulator_path, verilator_path = elaborate(sys, verilog='verilator')

    raw = utils.run_simulator(simulator_path)
    check(raw)

    raw = utils.run_verilator(verilator_path)
    check(raw)


if __name__ == '__main__':
    test_fifo_valid()
