import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, kmp):
        idx = RegArray(Int(13), 1)
        sram = SRAM(32, 8192, 'init.hex')
        read = idx[0] < Int(13)(8191)
        sram.build(Bits(1)(0), read, idx[0], Bits(32)(0), kmp)
        with Condition(read):
            sram.bound.async_called()
            idx[0] = idx[0] + Int(13)(1)
        with Condition(~read):
            log('done')
            finish()
        return idx

class KMP(Module):

    def __init__(self):
        super().__init__(ports={'rdata': Port(Bits(32))})
        self.res = RegArray(Int(32), 1)
        self.pattern = RegArray(Bits(32), 1)
        self.name = "kmp"

    @module.combinational
    def build(self):
        rdata = self.pop_all_ports(False)
        last = RegArray(Bits(32), 1, initializer=[0])
        a = last[0]
        b = rdata
        x = a.concat(b)
        ONE = Int(32)(1)
        ZERO = Int(32)(0)
        delta = (x[8:39] == self.pattern[0]).select(ZERO, ONE) + \
                (x[16:47] == self.pattern[0]).select(ZERO, ONE) + \
                (x[24:55] == self.pattern[0]).select(ZERO, ONE) + \
                (x[32:63] == self.pattern[0]).select(ZERO, ONE)
        self.res[0] = self.res[0] + delta
        last[0] = rdata

def test_kmp():
    sys = SysBuilder('kmp')
    with sys:
        kmp = KMP()
        kmp.build()
        sys.expose_on_top(kmp.res)

        driver = Driver()
        idx = driver.build(kmp)
        sys.expose_on_top(idx)


    config = backend.config(
            sim_threshold=100000,
            idle_threshold=100000,
            resource_base=f'{utils.repo_path()}/examples/kmp/input',
            verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    # check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        # check(raw)

if __name__ == "__main__":
    test_kmp()
