import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils


class SRAM(Memory):

    @module.constructor
    def __init__(self, init_file):
        super().__init__(width=32, depth=1024, latency=(1, 1), init_file=init_file)

    @module.combinational
    def build(self):
        super().build()
        read = ~self.we
        with Condition(read):
            rdata = self.rdata.bitcast(Int(32))
            k = Int(32)(128)
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
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        we = v[0:0]
        plused = v + Int(32)(1)
        waddr = plused[0:9]
        raddr = v[0:9]
        addr = we.select(waddr, raddr).bitcast(Int(10))
        memory.async_called(we = we.bitcast(Int(1)), addr = addr, wdata = v.bitcast(Bits(32)))
        cnt[0] = plused


def impl(sys_name, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        # Build the SRAM module
        memory = SRAM(init_file)
        memory.wait_until()
        memory.build()
        # Build the driver
        driver = Driver()
        driver.build(memory)

    config = backend.config(sim_threshold=200, idle_threshold=200, resource_base=resource_base, verilog=utils.verilator_path())

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)

    for line in raw.splitlines():
        if '[sram]' in line:
            toks = line.split()
            c = int(toks[-1])
            b = int(toks[-3])
            a = int(toks[-5])
            assert c % 2 == 1 or a == 0, f'Expected odd number or zero, got {line}'
            assert c == a + b, f'{a} + {b} = {c}'

def test_memory():
    impl('memory', None, None)

def test_memory_init():
    impl('memory_init', 'init.hex', f'{utils.repo_path()}/python/unit-tests/resources')

if __name__ == "__main__":
    test_memory()
    test_memory_init()
