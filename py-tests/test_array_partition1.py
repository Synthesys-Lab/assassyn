import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(explicit_fifo=True)
    
    @module.combinational
    def build(self):
        a = RegArray(Int(32), 4, attr=[Array.FULLY_PARTITIONED], initializer=[1, 2, 3, 4])
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        new_v = v + Int(32)(1)
        cnt[0] = new_v
        idx0 = v[0: 1]
        idx1 = new_v[0: 1]
        a[idx0] = (v * v)[0: 31].bitcast(Int(32))
        a[idx1] = new_v + new_v
        a_sum = a[idx0] + a[idx1]
        log("a[idx0] + a[idx1] = {}", a_sum)

        
def test_array_partition1():
    sys = SysBuilder('array_partition1')
    with sys:
        driver = Driver()
        driver.build()

    print(sys)

    simulator_path = elaborate(sys)
    
    raw = utils.run_simulator(simulator_path)
    print(raw)

    a = [1, 2, 3, 4]
    for i in raw.split('\n'):
        if "a[idx0] + a[idx1]" in i:
            line_toks = i.split()
            cycle = int(line_toks[2][1:-4])
            idx0 = cycle % 4
            idx1 = (cycle + 1) % 4
            expect = a[idx0] + a[idx1]
            assert expect == int(line_toks[-1]), f"@cycle: {cycle}"
            a[idx0] = cycle * cycle
            a[idx1] = (cycle + 1) * 2
    
if __name__ == '__main__':
    test_array_partition1()
