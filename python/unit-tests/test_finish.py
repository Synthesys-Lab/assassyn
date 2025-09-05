from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Driver(Module):

    def __init__(self):
        super().__init__(
            ports={} ,
        )

    @module.combinational
    def build(self):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        log('cnt: {}', cnt[0]);
        with Condition(cnt[0] >= Int(32)(50)):
            finish()

def check(raw):
    expected = 0
    for i in raw.split('\n'):
        if '[Driver]' in i or '[driver]' in i:
            assert int(i.split()[-1]) <= 50
            expected += 1
    assert expected == 50 or expected == 51, f'{expected} not in [50, 51]'

def test_driver():
    sys = SysBuilder('finish')
    with sys:
        driver = Driver()
        driver.build()

    print(sys)

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_driver()
