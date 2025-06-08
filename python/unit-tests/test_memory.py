import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils


class MemUser(Module):

    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width))}, 
        )

    @module.combinational
    def build(self):
        width = self.rdata.dtype.bits
        rdata = self.pop_all_ports(False)
        rdata = rdata.bitcast(Int(width))
        k = Int(width)(128)
        delta = rdata + k
        log('{} + {} = {}', rdata, k, delta)


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user):
        cnt = RegArray(Int(width), 1)
        v = cnt[0]
        we = v[0:0]
        re = ~we
        plused = v + Int(width)(1)
        waddr = plused[0:8]
        raddr = v[0:8]
        addr = we.select(waddr, raddr).bitcast(Int(9))
        cnt[0] = plused
        sram = SRAM(width, 512, init_file)
        sram.build(we, re, addr, v.bitcast(Bits(width)), user)
        with Condition(re):
            sram.bound.async_called()


def check(raw):
    for line in raw.splitlines():
        if '[memuser' in line:
            toks = line.split()
            c = int(toks[-1])
            b = int(toks[-3])
            a = int(toks[-5])
            assert c % 2 == 1 or a == 0, f'Expected odd number or zero, got {line}'
            assert c == a + b, f'{a} + {b} = {c}'


def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        user = MemUser(width)
        user.build()
        # Build the driver
        driver = Driver()
        driver.build(width, init_file, user)

    config = backend.config(sim_threshold=200, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    # if utils.has_verilator():
    #     raw = utils.run_verilator(verilator_path)
    #     check(raw)


def test_memory():
    impl('memory', 32, None, None)

def test_memory_init():
    impl('memory_init', 32, 'init_1.hex', f'{utils.repo_path()}/python/unit-tests/resources')

def test_memory_wide():
    impl('memory_wide', 256, None, None)

if __name__ == "__main__":
    test_memory()
    test_memory_init()
    test_memory_wide()
