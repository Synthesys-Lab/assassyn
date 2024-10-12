import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.expr import Bind

class Sub(Module):
    
    def __init__(self):
        super().__init__(ports={
            'a': Port(Int(32)),
            'b': Port(Int(32)),
        })

    @module.combinational
    def build(self):
        a_valid = self.a.valid()
        b_valid = self.b.valid()
        both_valid = a_valid & b_valid
        wait_until(both_valid)
        a, b = self.pop_all_ports(False)
        c = a - b
        log("sub: {} - {} = {}", a, b, c);


class Lhs(Module):
    
    def __init__(self):
        super().__init__({
            'v': Port(Int(32)),
        })

    @module.combinational
    def build(self, sub: Sub):
        v = self.pop_all_ports(True)
        rhs = sub.bind(a = v)
        return rhs

class Driver(Module):
    
        def __init__(self):
            super().__init__(ports={})

        @module.combinational
        def build(self, rhs: Bind, lhs: Lhs):
            cnt = RegArray(Int(32), 1)
            k = cnt[0]
            v = k + Int(32)(1)
            cnt[0] = v
            vv = v + v
            lhs.async_called(v = vv)
            rhs.async_called(b = v)
 

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
        sub.build()
        
        lhs = Lhs()
        rhs = lhs.build(sub)

        driver = Driver()
        driver.build(rhs, lhs)

    print(sys)

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_fifo_valid()
