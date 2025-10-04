from assassyn.frontend import *
from assassyn.test import run_test
from assassyn import utils


class Driver(Module):

    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self, lhs: Module, rhs: Module):
        cnt = RegArray(UInt(32), 1)
        v = cnt[0]
        (cnt & self)[0] <= cnt[0] + UInt(32)(1)
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

def build_system():
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

def test_toposort():
    run_test('topo_sort', build_system, check_raw,
             sim_threshold=100, idle_threshold=100)

if __name__ == '__main__':
    test_toposort()
