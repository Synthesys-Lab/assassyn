import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils


class SRAM(Memory):

    @module.constructor
    def __init__(self, init_file, width):
        super().__init__(width=width, depth=1024, latency=(1, 1), init_file=init_file)

    @module.combinational
    def build(self, width):
        super().build()
        read = ~self.we
        with Condition(read):
            rdata = self.rdata.bitcast(Int(width))
            k = Int(width)(128)
            delta = rdata + k
            log('{} + {} = {}', rdata, k, delta)

    @module.wait_until
    def wait_until(self):
        return self.validate_all_ports()


class Driver(Module):

    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, memory: SRAM):
        cnt = RegArray(Int(memory.width), 1)
        v = cnt[0]
        we = v[0:0]
        plused = v + Int(memory.width)(1)
        waddr = plused[0:9]
        raddr = v[0:9]
        addr = we.select(waddr, raddr).bitcast(Int(10))
        memory.async_called(
                we = we.bitcast(Int(1)),
                addr = addr,
                wdata = v.bitcast(Bits(memory.width)))
        cnt[0] = plused

def check(raw):
    for line in raw.splitlines():
        if '[sram]' in line:
            toks = line.split()
            c = int(toks[-1])
            b = int(toks[-3])
            a = int(toks[-5])
            assert c % 2 == 1 or a == 0, f'Expected odd number or zero, got {line}'
            assert c == a + b, f'{a} + {b} = {c}'


def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        # Build the SRAM module
        memory = SRAM(init_file, width)
        memory.wait_until()
        memory.build(width)
        # Build the driver
        driver = Driver()
        driver.build(memory)

    config = backend.config(sim_threshold=200, idle_threshold=200, resource_base=resource_base, verilog=utils.has_verilator())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        check(raw)


def test_memory():
    impl('memory', 32, None, None)

def test_memory_init():
    impl('memory_init', 32, 'init.hex', f'{utils.repo_path()}/python/unit-tests/resources')

def test_memory_wide():
    impl('memory_wide', 256, None, None)

if __name__ == "__main__":
    #test_memory()
    #test_memory_init()
    test_memory_wide()
