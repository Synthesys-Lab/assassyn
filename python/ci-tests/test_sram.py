import assassyn
from assassyn.frontend import *
from assassyn.test import run_test
from assassyn import utils
from assassyn.ir.module.downstream import Downstream, combinational


class MemUser(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, rdata: RegArray):
        width = rdata.scalar_ty.bits
        rdata_val = rdata[0].bitcast(Int(width))
        k = Int(width)(128)
        delta = rdata_val + k
        log('{} + {} = {}', rdata_val, k, delta)


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
        (cnt & self)[0] <= plused
        sram = SRAM(width, 512, init_file)
        sram.build(we, re, addr, v.bitcast(Bits(width)))
        user.async_called()
        return sram


def check(raw):
    for line in raw.splitlines():
        if '[memuser' in line:
            toks = line.split()
            c = int(toks[-1])
            b = int(toks[-3])
            a = int(toks[-5])
            assert c % 2 == 1 or a == 0, f'Expected odd number or zero, got {line}'
            assert c == a + b, f'{a} + {b} = {c}'


def test_memory():
    def top():
        user = MemUser()
        driver = Driver()
        sram = driver.build(32, None, user)
        user.build(sram.dout)

    run_test('memory', top, check, sim_threshold=200, idle_threshold=200)

def test_memory_init():
    def top():
        user = MemUser()
        driver = Driver()
        sram = driver.build(32, 'init_1.hex', user)
        user.build(sram.dout)

    run_test('memory_init', top, check,
             sim_threshold=200, idle_threshold=200,
             resource_base=f'{utils.repo_path()}/python/ci-tests/resources')

def test_memory_wide():
    def top():
        user = MemUser()
        driver = Driver()
        sram = driver.build(256, None, user)
        user.build(sram.dout)

    run_test('memory_wide', top, check, sim_threshold=200, idle_threshold=200)

if __name__ == "__main__":
    test_memory()
    test_memory_init()
    test_memory_wide()
