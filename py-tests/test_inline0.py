import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


def adder(a, b):
    c = a + b
    log("adder: {} + {} = {}", a, b, c)

class Driver(Module):


    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self):
        cnt = RegArray(Int(32), 1)
        k = cnt[0]
        v = k + Int(32)(1)
        cnt[0] = v
        adder(v, v)

def test_inline0():
    sys = SysBuilder('inline0')
    with sys:
        driver = Driver()
        driver.build()

    print(sys)

    simulator_path = elaborate(sys)
    raw = utils.run_simulator(simulator_path)

    print(raw)
    
    for i in raw.split('\n'):
        if f'[{driver.synthesis_name().lower()}]' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)

if __name__ == '__main__':
    test_inline0()
