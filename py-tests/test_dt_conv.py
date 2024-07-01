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
        i32 = Int(32)(1)
        b32 = i32.bitcast(Bits(32))
        i64z = i32.zext(Int(64))
        i64s = i32.sext(Int(64))

        b32_array = RegArray(Bits(32), 1)
        b32_array[0] = b32
        i64z_array = RegArray(Int(64), 1)
        i64s_array = RegArray(Int(64), 1)
        i64z_array[0] = i64z
        i64s_array[0] = i64s
        log("{} {} {}", b32_array[0], i64z_array[0], i64s_array[0]);


        
def test_dt_conv():
    
    sys = SysBuilder('dt_conv')
    with sys:
        driver = Driver()
        driver.build()

    print(sys)

    simulator_path = elaborate(sys)

    raw = utils.run_simulator(simulator_path)

    print(raw)
    
if __name__ == '__main__':
    test_dt_conv()
