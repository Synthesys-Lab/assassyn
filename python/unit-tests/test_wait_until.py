import assassyn
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Squarer(Module):

    def __init__(self):
        super().__init__(
            ports={'a': Port(Int(32))},
        )

    @module.combinational
    def build(self):
        a = self.pop_all_ports(True)
        b = a * a
        log("Multiplier: {} ^ 2 = {}", a, b)

class Agent(Module):

    def __init__(self):
        super().__init__(
            ports={'a': Port(Int(32))},
        )

    @module.combinational
    def build(self, lock, sqr: Squarer):
        wait_until(lock[0])
        a = self.pop_all_ports(False)
        sqr.async_called(a = a)

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

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
        agent.build(lock, sqr)

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
