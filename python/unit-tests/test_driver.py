from assassyn.frontend import *
from assassyn.test import run_test

class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        cnt = RegArray(UInt(32), 1)
        (cnt & self)[0] <= cnt[0] + UInt(32)(1)
        log('cnt: {}', cnt[0]);

def check(raw):
    print(raw)
    expected = 0
    for i in raw.split('\n'):
        if 'cnt:' in i:
            assert int(i.split()[-1]) == expected
            expected += 1
    assert expected == 100, f'{expected} != 100'

def test_driver():
    def top():
        driver = Driver()
        driver.build()

    run_test('driver', top, check)


if __name__ == '__main__':
    test_driver()
