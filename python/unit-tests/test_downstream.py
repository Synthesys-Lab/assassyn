from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Driver(Module):

    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, lhs: Module, rhs: Module):
        cnt = RegArray(UInt(32), 1)
        v = cnt[0]
        cnt[0] = cnt[0] + UInt(32)(1)
        lhs.async_called(data=v)
        rhs.async_called(data=v)


class ForwardData(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.data = Port(Int(32))

    @module.combinational
    def build(self):
        return self.data


class Adder(Downstream):

    @downstream.constructor
    def __init__(self, a, b):
        super().__init__()
        true = UInt(1)(1)
        self.a = Optional(a, true)
        self.b = Optional(b, true)

    @downstream.combinational
    def build(self):
        a = self.a
        b = self.b
        c = a.unwrap_or(Int(32)(0)) + b.unwrap_or(Int(32)(0))

def test_driver():
    sys = SysBuilder('driver')
    with sys:
        driver = Driver()
        lhs = ForwardData()
        rhs = ForwardData()
        adder = Adder(lhs.data, rhs.data)

        driver.build(lhs, rhs)
        adder.build()

    print(sys)


if __name__ == '__main__':
    test_driver()
