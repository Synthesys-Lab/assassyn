import pytest

from assassyn.frontend import *
from assassyn.test import run_test

class Driver(Module):

    def __init__(self):
        super().__init__(
            ports={} ,
        )

    @module.combinational
    def build(self):
        log("Hello, World!")


def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        cnt += "Hello, World!" in i
    assert cnt == 100, "Hello, World! not found in raw output"


def test_helloworld():
    def top():
        Driver().build()

    run_test('helloworld', top, check_raw)


if __name__ == '__main__':
    test_helloworld()
