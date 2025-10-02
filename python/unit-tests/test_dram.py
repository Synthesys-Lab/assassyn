import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
from assassyn.ir.module.downstream import Downstream, combinational


class handle_response(Module):

    def __init__(self, width):
        ports = {
            'kind_we': Port(Bits(1)),
            'kind_re': Port(Bits(1)),
            'write_success': Port(Bits(1)),
            'mem': Port(Bits(32)),
        }
        super().__init__(
            ports=ports,
        )
        self.width = width

    @module.combinational
    def build(self):
        kind_we = self.kind_we.pop()
        kind_re = self.kind_re.pop()
        write_success = self.write_success.pop()
        # data = mem_resp(self)
        with Condition(kind_re):
            mem = self.mem.pop()

        handle = handle_handler(self.width, kind_we, kind_re, write_success)
        handle.build()

class handle_handler(Downstream):
    def __init__(self, width, kind_we, kind_re, write_success):
        super().__init__()
        self.width = width
        self.kind_we = kind_we
        self.kind_re = kind_re
        self.write_success = write_success

    @combinational
    def build(self):
        with Condition(self.kind_we):
            log('Write request received, it is success or false (1 or 0): {}', self.write_success)
        with Condition(self.kind_re):
            data = mem_resp(self)
            k = Int(self.width)(128)
            delta = data + k
            log('{} + {} = {}', data, k, delta)

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, handle_response):
        cnt = RegArray(Int(width), 1)
        v = cnt[0]
        we = v[0:0]
        re = ~we
        plused = v + Int(width)(1)
        waddr = plused[0:8]
        raddr = v[0:8]
        addr = we.select(waddr, raddr).bitcast(Int(9))
        cnt[0] = plused
        dram = DRAM(width, 512, init_file)
        dram.build(we, re, addr, v.bitcast(Bits(width)), handle_response)
        has_resp = has_mem_resp(dram)
        with Condition(we | has_resp):
            dram.bound.async_called()


def check(raw):
    for line in raw.splitlines():
        if '[handle_handler' in line:
            toks = line.split()
            a_string = toks[-12] if len(toks) >= 12 else '0'
            if a_string != 'Write':      
                c = int(toks[-1])
                b = int(toks[-3])
                a = int(toks[-5])
                assert c % 2 == 1 or a == 0, f'Expected odd number or zero or write operation, got {line}'


def impl(sys_name, width, init_file, resource_base):
    sys = SysBuilder(sys_name)
    with sys:
        response = handle_response(width)
        response.build()
        # Build the driver
        driver = Driver()
        driver.build(width, init_file, response)

    config = backend.config(sim_threshold=200, idle_threshold=200, resource_base=resource_base, verilog=False)

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    # if utils.has_verilator():
    #     raw = utils.run_verilator(verilator_path)
    #     check(raw)


def test_memory():
    impl('memory', 32, None, None)

if __name__ == "__main__":
    test_memory()