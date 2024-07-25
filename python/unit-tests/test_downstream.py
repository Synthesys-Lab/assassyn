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


class Downstream(Module):

    @module.constructor
    def __init__(self):
        super().__init__(is_downstream=True)

    @module.combinational
    def build(self, a: Optional, b: Optional):
        c = a.unwrap_or(Int(32)(0)) + b.unwrap_or(Int(32)(0))

def test_driver():
    sys = SysBuilder('driver')
    with sys:
        driver = Driver()
        downstream = Downstream()
        lhs = ForwardData()
        rhs = ForwardData()

        driver.build(lhs, rhs)
        true = UInt(1)(1)
        a = Optional(lhs.build(), true)
        b = Optional(rhs.build(), true)
        downstream.build(a, b)
        
    print(sys)


if __name__ == '__main__':
    test_driver()
