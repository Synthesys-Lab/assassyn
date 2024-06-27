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
    def build(self, lock: Array, sqr: Squarer):
        with self.wait_until(lock):
            sqr.async_called(a = self.a)

class Driver(Module):

    @module.constructor
    def __init__(self):
        pass

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

    print(sys)

    simulator_path = elaborate(sys, sim_threshold=200, idle_threshold=200)

    raw = utils.run_simulator(simulator_path)

    print(raw)

    for i in raw.split('\n'):
        if 'squarer' in i:
            toks = i.split()
            cycle = int(toks[2][1:-4])
            assert cycle % 4 in [2, 3], cycle
            value = int(toks[-5])
            res = int(toks[-1])
            assert res == value * value


if __name__ == '__main__':
    test_wait_until()
