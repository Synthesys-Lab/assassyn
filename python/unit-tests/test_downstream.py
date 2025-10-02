from assassyn.frontend import *
from assassyn.test import run_test


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

def build_system():
    driver = Driver()
    lhs = ForwardData()
    rhs = ForwardData()
    a = lhs.build()
    b = rhs.build()
    adder = Adder()

    driver.build(lhs, rhs)
    adder.build(a, b)

def test_downstream():
    run_test('downstream', build_system, check_raw, sim_threshold=100, idle_threshold=100)

if __name__ == '__main__':
    test_downstream()
