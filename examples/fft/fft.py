import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
from assassyn.expr import Bind
from assassyn.backend import elaborate
import os
import shutil

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

I_MAX = 5
FFT_SIZE = 16


class Loop_user(Module):

    def __init__(self):
        ports={
            
        }
        super().__init__(
            ports=ports ,
        )

    @module.combinational
    def build(self, span: Array, odd: Array):
        p = RegArray(Int(32), 1)
        p[0] = (span[0][0:15].bitcast(Int(16)) * Int(16)(FFT_SIZE+1)).bitcast(Int(32)) + odd[0]
        log("p: {} = span_{}, odd_{}", p[0], span[0], odd[0])

class External_loop(Module):
    def __init__(self):
        super().__init__(
            ports={'In_full_flag': Port(Bits(1))},
        )

    @module.combinational
    def build(self):
        In_full_flag = self.pop_all_ports(True)
        span = RegArray(Int(32), 1)
        with Condition(In_full_flag == Bits(1)(1)):
            con = Bits(1)(0)
            con = span[0] > Int(32)(0)
            full_flag = span[0] == Int(32)(I_MAX)
            span[0] = con.select((span[0].bitcast(Int(32)) >> Int(32)(1)) , Int(32)(0))
        
        return span

class Internal_loop(Module):
    def __init__(self):
        super().__init__(
            ports={ },
        ) 
        
    @module.combinational
    def build(self):
        
        j = RegArray(Int(32), 1, initializer=[0])
        span = RegArray(Int(32), 1, initializer=[FFT_SIZE >> 1])
        
        with Condition(span[0] == Int(32)(0)):
            finish()
        
        con = Bits(1)(0)
        con = j[0] == Int(32)(FFT_SIZE)
        j[0] = con.select(Int(32)(0), (j[0].bitcast(Int(32)) + Int(32)(1)))
        
        span[0] = con.select((span[0].bitcast(Int(32)) >> Int(32)(1)), span[0])
        
        # outter_loop.async_called( In_full_flag = full_flag.bitcast(Bits(1)))
        return span, j



class Driver(Module):
    def __init__(self):
        super().__init__(
            ports={},
        )
 
    @module.combinational
    def build(self, inner_loop: Internal_loop, user: Loop_user):
        inner_loop.async_called()
        user.async_called()

def test_fft():
    sys =  SysBuilder('fft')
    with sys:
        # external_loop = External_loop()
        # span = external_loop.build()

        internal_loop = Internal_loop()
        span, j = internal_loop.build()

        loop_user = Loop_user()
        loop_user.build(span, j)

        driver = Driver()
        driver.build(internal_loop, loop_user)
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=200,
        idle_threshold=200,
        
    )
    simulator_path, verilator_path = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    
    if verilator_path:
        raw = utils.run_verilator(verilator_path)

if __name__ == '__main__':
    test_fft()