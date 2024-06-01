from assassyn import *

sys = SysBuilder('async_call')
with sys:
    pass

#class Adder(Module):
#
#    def __init__(self):
#        self.a = Port(Int(32))
#        self.b = Port(Int(32))
#
#    @assassyn.wait_until
#    def ready(self):
#        return self.a.valid() & self.b.valid()
#
#    @assassyn.module_builder
#    def build(self):
#        self.c = self.a + self.b
#
#
#class Driver(assassyn.Module):
#
#    def __init__(self):
#        pass
#
#    @assassyn.module_builder
#    def build(self, adder: Adder):
#        cnt = Array(UInt(32), 1)
#        cnt[0] = cnt[0] + UInt(32)(1)
#        adder.call(a = cnt[0], b = cnt[0])
#
#
#with SysBuilder() as sys:
#
#    driver = Driver()
#    adder = Adder()
#
#    adder.build()
#    driver.build(adder)
