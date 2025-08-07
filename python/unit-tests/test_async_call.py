from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
import assassyn

class Adder(Module):

    def __init__(self):
        super().__init__(
            ports={
                'a': Port(Int(32)),
                'b': Port(Int(32)),
            },
        )

    @module.combinational
    def build(self):
        a, b = self.pop_all_ports(True)
        c = a + b
        log("Adder: {} + {} = {}", a, b, c)

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, adder: Adder):
        cnt = Int(32)(0)
        cnt = cnt + Int(32)(1)
        cond = cnt < Int(32)(100)
        log('Pushes {}', cnt)
        with Condition(cond):
            adder.async_called(a = cnt, b = cnt)

def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        if 'Adder:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 100, f'cnt: {cnt} != 100'


def test_async_call():
    # NOTE: The name of the system should be unique within all the testcases,
    # because we currently have no locks to exclusively own a folder, under the
    # context of multi-thread testing.
    sys = SysBuilder('async_call')
    with sys:
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    config = assassyn.backend.config(
            verilog=utils.has_verilator(),
            sim_threshold=200,
            idle_threshold=200,
            random=True)

    simulator_path, verilator_path = elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    check_raw(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check_raw(raw)


if __name__ == '__main__':
    test_async_call()
