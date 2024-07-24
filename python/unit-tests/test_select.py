import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self):
        rng0 = RegArray(Int(32), 1)
        rng1 = RegArray(Int(32), 1)

        v0 = rng0[0]
        v1 = rng1[0]

        v0 = v0 * Int(32)(12345)
        v1 = v1 * Int(32)(54321)
        
        rand0 = v0 + Int(32)(67890)
        rand1 = v1 + Int(32)(9876)

        rand0 = rand0[0: 31].bitcast(Int(32))
        rand1 = rand1[0: 31].bitcast(Int(32))

        gt = rand0 > rand1
        mux = gt.select(rand0, rand1)

        log("select: {} >? {} = {}", rand0, rand1, mux)
 

def check(raw: str):
    for i in raw.splitlines():
        if 'select:' in i:
            a = i.split()[-5]
            b = i.split()[-3]
            c = i.split()[-1]
            assert max(int(a), int(b)) == int(c)

        
def test_select():
    sys = SysBuilder('select')
    with sys:
        driver = Driver()
        driver.build()

    simulator_path, verilator_path = elaborate(sys, verilog='verilator')

    raw = utils.run_simulator(simulator_path)
    check(raw)

    raw = utils.run_verilator(verilator_path)
    check(raw)


if __name__ == '__main__':
    test_select()
