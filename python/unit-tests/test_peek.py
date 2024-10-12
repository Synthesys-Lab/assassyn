import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Peeker(Module):
     
    def __init__(self):
        super().__init__(
            ports={'data': Port(Int(32))}, 
        )

    @module.combinational
    def build(self):
        data_peek   = self.data.peek()
        data_pop    = self.data.pop()
        log("peek: {} pop: {}", data_peek, data_pop)

        
class Driver(Module):
    
    def __init__(self):
            super().__init__(ports={})
            
    @module.combinational
    def build(self, peeker: Module):
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        peeker.async_called(data = v)
        v = v + Int(32)(1)
        cnt[0] = v

def check(raw):
    for i in raw.split('\n'):
        if "peek:" in i:
            line_toks = i.split()
            assert line_toks[-1] == line_toks[-3], \
                f"peek: {line_toks[-3]}, pop: {line_toks[-1]}"

def test_peek():
    sys = SysBuilder("peek")
    with sys:
        peeker = Peeker()
        peeker.build()

        driver = Driver()
        driver.build(peeker)

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())
    
    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)

if __name__ == '__main__':
    test_peek()
        
