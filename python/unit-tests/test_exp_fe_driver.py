# Test the experimental frontend by replicating ./test_driver.py

from assassyn.frontend import *
from assassyn.test import run_test
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
    run_test('driver_exp', driver_factory, check)


if __name__ == '__main__':
    test_exp_fe_driver()
