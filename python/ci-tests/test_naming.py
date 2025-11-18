from assassyn.frontend import *
from assassyn.test import run_test

class Driver(Module):
    """Issues fetch requests for every word in the program image."""

    def __init__(self):
        super().__init__(ports={})
        self.name = "Driver"

    @module.combinational
    def build(self):
        log("{}",Bits(1)(0) & Bits(1)(1))

def check(raw):
    print(raw)
    expected = 0
    for i in raw.split('\n'):
        if 'Driver' in i:
            expected += 1
    assert expected == 100, f'{expected} != 100'

def test_driver():
    def top():
        driver = Driver()
        driver.build()

    run_test('naming', top, check)


if __name__ == '__main__':
    test_driver()
