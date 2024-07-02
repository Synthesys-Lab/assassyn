import pytest

from assassyn import utils
from assassyn.backend import elaborate
from assassyn.frontend import *

class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(explicit_fifo=True)

    @module.combinational
    def build(self):
        a = RegArray(Int(32), 4, attr=[Array.FULLY_PARTITIONED])
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        cnt[0] = v + Int(32)(1)
        a[0] = v
        a[1] = v
        a[2] = v
        a[3] = v
        a_sum = a[0] + a[1] + a[2] + a[3]
        log("sum(a[:]) = {}", a_sum)


def test_array_partition0():
    sys = SysBuilder('array_partition0')
    with sys:
        driver = Driver()
        driver.build()

    print(sys)

    simulator_path = elaborate(sys)

    raw = utils.run_simulator(simulator_path)

    print(raw)

    for i in raw.split('\n'):
        if 'sum(a[:])' in i:
            line_toks = i.split()
            cycle = int(line_toks[2][1:-4])
            assert (int(line_toks[-1]) % 4) == 0
            assert max(cycle - 1, 0) * 4 == int(line_toks[-1])

if __name__ == '__main__':
    test_array_partition0()
