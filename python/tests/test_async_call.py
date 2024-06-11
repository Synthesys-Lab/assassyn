from assassyn import *


class Adder(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))
        self.b = Port(Int(32))

    @module.combinational
    def build(self):
        a = self.a.pop()
        b = self.b.pop()
        a + b

class Driver(Module):

    def __init__(self):
        pass

    @module.combinational
    def build(self, adder: Adder):
        cnt = Array(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        adder.async_called(a = cnt[0], b = cnt[0])


sys = SysBuilder('async_call')
with sys:
    adder = Adder()
    adder.build()

    driver = Driver()
    driver.build(adder)

