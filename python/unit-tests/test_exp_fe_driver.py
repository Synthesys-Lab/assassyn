# Test the experimental frontend by replicating ./test_driver.py

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.experimental.frontend import pipeline


@pipeline.factory
def driver_factory() -> pipeline.Stage:
    def driver():
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        log('cnt: {}', cnt[0])
    return driver


def check(raw):
    expected = 0
    for i in raw.split('\n'):
        if 'cnt:' in i:
            assert int(i.split()[-1]) == expected
            expected += 1
    assert expected == 100, f'{expected} != 100'


def test_exp_fe_driver():
    sys = SysBuilder('driver')
    with sys:
        driver_factory()

    print(sys)

    simulator_path, verilator_path = elaborate(sys, verilog=utils.has_verilator())

    raw = utils.run_simulator(simulator_path)
    check(raw)

    if verilator_path:
        raw = utils.run_verilator(verilator_path)
        check(raw)


if __name__ == '__main__':
    test_exp_fe_driver()
