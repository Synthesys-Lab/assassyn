from assassyn.experimental.frontend.factory import factory
from assassyn.experimental.frontend.module import ModuleFactory
from assassyn.frontend import Module, RegArray, UInt, log
from assassyn.test import run_test


@factory(Module)
def driver_factory() -> ModuleFactory:
    def driver():
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        log('cnt: {}', cnt[0])
    return driver


def check(raw):
    expected = 0
    for line in raw.split('\n'):
        if 'cnt:' in line:
            assert int(line.split()[-1]) == expected
            expected += 1
    assert expected == 100, f'{expected} != 100'


def test_exp_fe_driver():
    def top():
        driver_factory()

    run_test('driver_exp', top, check)


if __name__ == '__main__':
    test_exp_fe_driver()
