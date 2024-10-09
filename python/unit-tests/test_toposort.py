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
        return self.pop_all_ports(True)


class Adder(Downstream):

    def __init__(self):
            super().__init__()

    @downstream.combinational
    def build(self, a: Value, b: Value, id: str):
        a = a.optional(UInt(32)(1))
        b = b.optional(UInt(32)(1))
        c = a + b
        log(f"downstream.{id}: {{}} + {{}} = {{}}", a, b, c)
        return c

def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        if 'downstream' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
            if 'downstream.e' in i:
                assert int(a) == int(b)
    assert cnt == 297, f'cnt: {cnt} != 297'

def test_toposort():
    sys = SysBuilder('topo_sort')
    with sys:
        driver = Driver()
        hs1 = ForwardData()
        hs2 = ForwardData()
        hs3 = ForwardData()
        hs4 = ForwardData()
        
        a = hs1.build()
        b = hs2.build()
        c = hs3.build()
        d = hs4.build()
        
        adder1 = Adder()
        adder1.name = 'adder1'
        adder2 = Adder()
        adder2.name = 'adder2'
        adder3 = Adder()
        adder3.name = 'adder3'

        driver.build(hs1, hs2)
        
        c = adder1.build(a, b, "c")
        d = adder2.build(a, b, "d")
        adder3.build(c, d, "e")

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
    test_toposort()
