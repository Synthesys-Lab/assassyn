from assassyn import *

class Driver(Module):

    @module.constructor
    def __init__(self):
        pass

    @module.combinational
    def build(self):
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        log('cnt: {}', cnt[0]);

sys = SysBuilder('async_call')
with sys:
    driver = Driver()
    driver.build()

print(sys)
