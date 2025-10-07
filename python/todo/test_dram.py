import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
from assassyn.ir.module.downstream import Downstream, combinational

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
        read_succ, write_succ = dram.build(we, re, addr, v.bitcast(Bits(width)), handle_response)
        return dram, read_succ, write_succ


class HandleResponse(Downstream):
    def __init__(self):
        super().__init__(ports={})
    
    @downstream.combinational
    def build(self, dram, read_succ, write_succ):
        assume(read_succ & write_succ)
        with Condition(has_mem_resp(dram)):
            resp = get_mem_resp(dram)
            data = resp[0:width]
            addr = resp[width:width+9]
            log('Read: {} @Addr: {}', data, addr)


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
        # Build the driver
        driver = Driver()
        dram, read_succ, write_succ = driver.build(width, init_file)
        handle_response = HandleResponse()
        handle_response.build(dram, read_succ, write_succ)

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