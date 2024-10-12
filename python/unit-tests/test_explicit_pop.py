# import like others
import pytest

from assassyn.backend import elaborate
from assassyn.frontend import *
from assassyn import utils 


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
        a = self.a.pop()
        b = self.b.pop()
        c = a + b
        log("adder: {} + {} = {}", a, b, c);

class Driver(Module):
    
    def __init__(self): 
        super().__init__(
            ports={} ,
        )  

    @module.combinational
    def build(self, adder: Adder):
        cnt = RegArray(Int(32), 1)
        k = cnt[0]
        v = k + Int(32)(1)
        cnt[0] = v
        adder.async_called(a = v, b = v)

def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        if f'adder:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 99, f'cnt: {cnt} != 100'

def test_explicit_pop():
    sys = SysBuilder('explicit_pop')
    with sys:
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check_raw(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check_raw(raw)


if __name__ == '__main__':
    test_explicit_pop()
