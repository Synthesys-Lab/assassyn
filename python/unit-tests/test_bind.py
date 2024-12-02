from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.expr import Bind

class Sub(Module):

    def __init__(self):
        ports={
            'sub_a': Port(Int(32)),
            'sub_b': Port(Int(32))
        }
        super().__init__(
            ports=ports ,
        )

    @module.combinational
    def build(self):
        a, b = self.pop_all_ports(False)
        c = a - b
        log("Subtractor: {} - {} = {}", a, b, c)

class Lhs(Module):
    def __init__(self):
        super().__init__(
            ports={'lhs_a': Port(Int(32))},
        )

    @module.combinational
    def build(self, sub: Sub):
        lhs_a = self.pop_all_ports(True)
        bound = sub.bind(sub_a = lhs_a)
        return bound

class Rhs(Module):
    def __init__(self):
        super().__init__(
            ports={'rhs_b': Port(Int(32))},
        ) 
        
    @module.combinational
    def build(self, sub: Bind):
        rhs_b = self.pop_all_ports(True)
        call = sub.async_called(sub_b = rhs_b)
        call.bind.set_fifo_depth(sub_a = 1, sub_b = 1)
        

class Driver(Module):
    def __init__(self):
        super().__init__(
            ports={},
        )
 
    @module.combinational
    def build(self, lhs: Lhs, rhs: Rhs):
        cnt = RegArray(Int(32), 1)
        cnt[0] = cnt[0] + Int(32)(1)
        v = cnt[0] * cnt[0]

        call_lhs = lhs.async_called(lhs_a = v[0: 31].bitcast(Int(32)))
        call_lhs.bind.set_fifo_depth(lhs_a = 1)
        call_rhs = rhs.async_called(rhs_b = cnt[0])
        call_rhs.bind.set_fifo_depth(rhs_b = 1)

def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        if f'Subtractor' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(b) - int(a) == int(c)
            cnt += 1
    assert cnt == 100 - 2, f'cnt: {cnt} != 98'

def test_bind():
    sys =  SysBuilder('bind')
    with sys:
        sub = Sub()
        sub.build()

        lhs = Lhs()
        aa_lhs = lhs.build(sub)

        rhs = Rhs()
        rhs.build(aa_lhs)

        driver = Driver()
        driver.build(lhs, rhs)

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check_raw(raw)
    
    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check_raw(raw)


if __name__ == '__main__':
    test_bind()
