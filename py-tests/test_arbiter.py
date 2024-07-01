import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils


class Square(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)
        self.a = Port(Int(32))

    @module.combinational
    def build(self):
        b = self.a * self.a
        log("adder: {} * {} = {}", self.a, self.a, b);


class Arbiter(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__(explicit_fifo=True, disable_arbiter_rewrite=True)
        self.a0 = Port(Int(32))
        self.a1 = Port(Int(32))

    @module.wait_until
    def wait_until(self):
        a0_valid = self.a0.valid()
        a1_valid = self.a1.valid()
        valid = a0_valid | a1_valid
        return valid

    @module.combinational
    def build(self):
        a0_valid = self.a0.valid()
        a1_valid = self.a1.valid()
        hot_valid = a1_valid.concat(a0_valid)
        grant_1h = RegArray(Bits(2), 1, initializer=[1])
        gv = grant_1h[0]
        gv_flip = ~gv
        hi = gv_flip & hot_valid
        lo = gv & hot_valid
        hi_nez = (hi != 0)
        # new_grant = 