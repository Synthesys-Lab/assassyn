import assassyn
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Driver(Module):
 
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, lhs: Module, rhs: Module):
        cnt = RegArray(UInt(32), 1)
        v = cnt[0]
        cnt[0] = cnt[0] + UInt(32)(1)
        lhs.async_called(data=v)
        rhs.async_called(data=v)


class ForwardData(Module):
    def __init__(self):
        super().__init__(
            ports={'data': Port(UInt(32))},
        ) 

    @module.combinational
    def build(self):
        data = self.pop_all_ports(True)
        return data


class Adder(Downstream):

    def __init__(self):
        super().__init__()

    @downstream.combinational
    def build(self, a: Value, b: Value):
        a = a.optional(UInt(32)(1))
        b = b.optional(UInt(32)(1))
        c = a + b
        log("downstream: {} + {} = {}", a, b, c)

def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        if 'downstream:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 99, f'cnt: {cnt} != 99'

def test_downstream():
    sys = SysBuilder('downstream')
    with sys:
        driver = Driver()
        lhs = ForwardData()
        rhs = ForwardData()
        a = lhs.build()
        b = rhs.build()
        adder = Adder()

        driver.build(lhs, rhs)
        adder.build(a, b)

    print(sys)

    config = assassyn.backend.config(
            verilog=utils.has_verilator(),
            sim_threshold=100,
            idle_threshold=100)

    simulator_path, verilator_path = elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check_raw(raw)
    
    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check_raw(raw)

if __name__ == '__main__':
    test_downstream()
