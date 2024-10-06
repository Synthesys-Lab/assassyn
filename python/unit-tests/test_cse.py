import pytest

import assassyn
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Adder(Module):
  
    def __init__(self):
        ports={
            'a': Port(Int(32)),
            'b': Port(Int(32))
        }
        super().__init__(
            ports=ports ,
        )

    @module.combinational
    def build(self):
        a, b = self.pop_all_ports(True)
        c = a + b
        log("Adder: {} + {} = {}", a, b, c)


class Driver(Module):
    def __init__(self):
        super().__init__(
            ports={} ,
        )
        
    @module.combinational
    def build(self, add: Adder):
        cnt = RegArray(Int(32), 1)
        cnt[0] = cnt[0] + Int(32)(1)
        cond = cnt[0] < Int(32)(100)
        with Condition(cond):
            add.async_called(a = cnt[0], b = cnt[0])
        with Condition(cond):
            log("Done")

def check(raw):
    cnt = 0
    for i in raw.split('\n'):
        if f'Adder' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 100, f'cnt: {cnt} != 100 - 1'


def test_cond_cse():
    sys = SysBuilder('cond_cse')
    with sys:
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    config = assassyn.backend.config(sim_threshold=200, idle_threshold=200, verilog=utils.has_verilator())
    simulator_path, verilator_path = elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_cond_cse()
