import subprocess

import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn.utils import run_simulator


class Adder(Module):
    a: Port

    def __init__(self):
        super().__init__({
                "a": Port(Int(32)),
        })

    @module.combinational
    def build(self):
        log("Before pop.")
        a = self.a.pop()
        log("After pop. a is: {}", a)


class Driver(Module):
    def __init__(self):
        super().__init__({})

    @module.combinational
    def build(self, adder: Adder):
        adder.async_called()


def top():
    sys = SysBuilder("optional")
    with sys:
        adder = Adder()
        driver = Driver()

        adder.build()
        driver.build(adder)

    return sys


def test_empty_fifo_pop_panics(capfd):
    sys = top()
    sim, _ = elaborate(sys, verbose=False, simulator=True, verilog=False)

    with pytest.raises(subprocess.CalledProcessError):
        run_simulator(sim)

    stdout, stderr = capfd.readouterr()
    combined = stdout + stderr
    assert "test_empty_pop.py" in combined
    assert "is trying to pop an empty FIFO" in combined

if __name__ == "__main__":
    test_empty_fifo_pop_panics()
