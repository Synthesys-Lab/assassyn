import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
import math

cachesize = 8

class MemUser(Module):

    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width))}, 
        )
        self.reg_accm = RegArray(Int(width), 1)
        self.name = "sram"
        
    @module.combinational
    def build(self):
        
        width = self.rdata.dtype.bits
        rdata = self.pop_all_ports(False)
        rdata = rdata.bitcast(Int(width))

        bitmask = Bits(cachesize)(0b10101010)
        data_joint = Bits(0)(0)

        for i in range(cachesize):
            offest = cachesize - i - 1
            data_joint = data_joint.concat(bitmask[offest:offest].select(rdata[offest*32:offest*32+31], Bits(32)(0)))

        log("Cacheline:\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", data_joint[224:255], data_joint[192:223], data_joint[160:191], data_joint[128:159], data_joint[96:127], data_joint[64:95], data_joint[32:63], data_joint[0:31])

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, width, init_file, user):
        cnt = RegArray(Int(width), 1)
        v = cnt[0]
        we = Int(1)(0)
        re = ~v[0:0]
        plused = v + Int(width)(1)
        raddr = v[0:8].bitcast(Int(9))
        
        shift = Int(9)(int(math.log2(cachesize)))
        addr_access = raddr >> shift
        
        cnt[0] = plused
        sram = SRAM(width, 512, init_file)
        sram.build(we, re, addr_access, v.bitcast(Bits(width)), user)
        with Condition(re):
            sram.bound.async_called()

def check(raw):
    for line in raw.splitlines():
        if '[sram]' in line:
            toks = line.split()
            c = int(toks[-1])
            b = int(toks[-3])
            a = int(toks[-5])
            assert a % 2 == 1 or c == 0, f'Expected odd number or zero, got {line}'
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
    print(raw)
    check(raw)

    if utils.has_verilator():
        raw = utils.run_verilator(verilator_path)
        check(raw)

def test_memory():
    impl('memory_init', 32*cachesize, 'init_multidata.hex', f'{utils.repo_path()}/python/unit-tests/resources')

if __name__ == "__main__":
        test_memory()
