import assassyn
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Squarer(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))

    @module.combinational
    def build(self):
        b = self.a * self.a
        log("Multiplier: {} ^ 2 = {}", self.a, b)

class Agent(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.a = Port(Int(32))

    @module.wait_until
    def wait_until(self, lock: Array):
        return lock[0]

    @module.combinational
    def build(self, sqr: Squarer):
        sqr.async_called(a = self.a)

class Driver(Module):

    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, agent: Agent, lock: Array):
        cnt = RegArray(Int(32), 1)
        is_odd = cnt[0][0:0]
        is_even = ~is_odd
        cnt[0] = cnt[0] + Int(32)(1)
        with Condition(is_odd):
            agent.async_called(a = cnt[0])
        with Condition(is_even):
            flip = ~lock[0]
            log('flip to {}', flip)
            lock[0] = flip

def parse_simulator_log(toks):
    cycle = utils.parse_simulator_cycle(toks)
    return cycle, int(toks[-5]), int(toks[-1])

def parse_verilator_log(toks):
    cycle = utils.parse_verilator_cycle(toks)
    return cycle, int(toks[-5]), int(toks[-1])

def check(raw, cycle_parser):
    for i in raw.splitlines():
        if 'Multiplier' in i:
            toks = i.split()
            cycle, value, res = cycle_parser(toks)
            assert (cycle - 1) % 4 in [2, 3], cycle
            assert res == value * value

def test_wait_until():
    sys = SysBuilder('wait_until')
    with sys:
        sqr = Squarer()
        sqr.build()

        lock = RegArray(UInt(1), 1)

        agent = Agent()
        agent.wait_until(lock)
        agent.build(sqr)

        driver = Driver()
        driver.build(agent, lock)


    config = assassyn.backend.config(sim_threshold=200, idle_threshold=200, verilog=utils.has_verilator())
    simulator_path, verilator_path = elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check(raw, parse_simulator_log)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw, parse_verilator_log)


if __name__ == '__main__':
    test_wait_until()
