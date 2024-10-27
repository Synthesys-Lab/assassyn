import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Driver(Module):
    
    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self):
        rng0 = RegArray(UInt(32), 1)
        rng1 = RegArray(UInt(32), 1)

        v0 = rng0[0]
        v1 = rng1[0]

        v0 = v0 * UInt(32)(12345)
        v1 = v1 * UInt(32)(54321)
        
        rand0 = v0 + UInt(64)(67890)
        rand1 = v1 + UInt(64)(9876)

        rand0 = rand0[0: 31].bitcast(UInt(32))
        rand1 = rand1[0: 31].bitcast(UInt(32))

        rng0[0] = rand0
        rng1[0] = rand1

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

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_select()
