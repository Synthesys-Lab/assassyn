import pytest

from assassyn.frontend import *
from assassyn.test import run_test

class Peeker(Module):

    def __init__(self):
        super().__init__(
            ports={'data': Port(Int(32))},
        )

    @module.combinational
    def build(self):
        data_peek   = self.data.peek()
        data_pop    = self.data.pop()
        log("peek: {} pop: {}", data_peek, data_pop)


class Driver(Module):

    def __init__(self):
            super().__init__(ports={})

    @module.combinational
    def build(self, peeker: Module):
        cnt = RegArray(Int(32), 1)
        v = cnt[0]
        peeker.async_called(data = v)
        v = v + Int(32)(1)
        (cnt & self)[0] <= v

def check(raw):
    for i in raw.split('\n'):
        if "peek:" in i:
            line_toks = i.split()
            assert line_toks[-1] == line_toks[-3], \
                f"peek: {line_toks[-3]}, pop: {line_toks[-1]}"

def test_peek():
    def top():
        peeker = Peeker()
        peeker.build()

        driver = Driver()
        driver.build(peeker)

    run_test("peek", top, check)

if __name__ == '__main__':
    test_peek()

