from assassyn.frontend import *
from assassyn.test import run_test
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
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        with Condition(is_odd):
            agent.async_called(a = cnt[0])
        with Condition(is_even):
            flip = ~lock[0]
            log('flip to {}', flip)
            (lock&self)[0] <= flip

def build_system():
    sqr = Squarer()
    sqr.build()

    lock = RegArray(Bits(1), 1)

    agent = Agent()
    agent.build(lock, sqr)

    driver = Driver()
    driver.build(agent, lock)

def check_output(raw):
    for i in raw.splitlines():
        if 'Multiplier' in i:
            toks = i.split()
            # Try parsing as simulator log first
            try:
                cycle = utils.parse_simulator_cycle(toks)
            except:
                cycle = utils.parse_verilator_cycle(toks)
            value = int(toks[-5])
            res = int(toks[-1])
            assert (cycle - 1) % 4 in [2, 3], cycle
            assert res == value * value

def test_wait_until():
    run_test('wait_until', build_system, check_output,
             sim_threshold=200, idle_threshold=200)


if __name__ == '__main__':
    test_wait_until()
