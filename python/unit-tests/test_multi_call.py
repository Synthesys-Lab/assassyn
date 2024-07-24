import pytest

from assassyn import utils
from assassyn.backend import elaborate
from assassyn.frontend import *


class Adder(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.add_a = Port(Int(32))
        self.add_b = Port(Int(32))

    @module.combinational
    def build(self):
        c = self.add_a + self.add_b
        log("adder: {} + {} = {}", self.add_a, self.add_b, c)

class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

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
            adder.async_called(add_a = odd, add_b = odd2)
            adder.async_called(add_a = even2, add_b = even)


def test_multi_call():

    sys = SysBuilder('multi_call')

    with sys:
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    print(sys)

    simulator_path = elaborate(sys, verilog=None)

    raw = utils.run_simulator(simulator_path)

    print(raw)

    for i in raw.split('\n'):
        if f'adder: ' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)

if __name__ == '__main__':
    test_multi_call()
