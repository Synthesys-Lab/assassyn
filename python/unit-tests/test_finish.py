from assassyn.frontend import *
from assassyn.test import run_test

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

def build_top():
    driver = Driver()
    driver.build()

def check(raw):
    expected = 0
    for i in raw.split('\n'):
        if '[Driver]' in i or '[driver]' in i:
            assert int(i.split()[-1]) <= 50
            expected += 1
    assert expected == 50 or expected == 51, f'{expected} not in [50, 51]'

def test_finish():
    run_test('finish', build_top, check)

if __name__ == '__main__':
    test_finish()
