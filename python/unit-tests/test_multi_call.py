import pytest

from assassyn import utils
from assassyn.backend import elaborate
from assassyn.frontend import *


class Adder(Module):
    
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
        c = a + b 
        log("adder: {} + {} = {}", a, b, c)

class Driver(Module):
    
    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self, adder: Adder):
        cnt = RegArray(Int(32), 1)
        k = cnt[0]
        v = k + Int(32)(1)
        even = (v * Int(32)(2))[0: 31].bitcast(Int(32))
        even2 = (even * Int(32)(2))[0: 31].bitcast(Int(32))
        odd = even + Int(32)(1)
        odd2 = (odd * Int(32)(2))[0: 31].bitcast(Int(32))
        cnt[0] = v
        is_odd = v[0: 0]
        with Condition(is_odd):
            adder.async_called(a = odd, b = odd2)
            adder.async_called(a = even2, b = even)

def check(raw):
    for i in raw.split('\n'):
        if f'adder: ' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)

def test_multi_call():

    sys = SysBuilder('multi_call')

    with sys:
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    print(sys)

    simulator_path, verilog_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilog_path:
        raw = utils.run_verilator(verilog_path)
        check(raw)


if __name__ == '__main__':
    test_multi_call()
