import pytest

from assassyn.frontend import *
from assassyn.test import run_test


class Squarer(Module):

    def __init__(self):
        super().__init__(
            ports={'a': Port(Int(32))},
            no_arbiter=True,
        )

    @module.combinational
    def build(self):
        a = self.pop_all_ports(validate=False)
        b = a * a
        log("adder: {} * {} = {}", a, a, b);


class Arbiter(Module):

    def __init__(self):
        ports={
            'a0': Port(Int(32)),
            'a1': Port(Int(32))
        }
        super().__init__(
            ports=ports,
            no_arbiter=True,
        )

    @module.combinational
    def build(self, sqr: Squarer):
        a0_valid = self.a0.valid()
        a1_valid = self.a1.valid()
        valid = a0_valid | a1_valid
        wait_until(valid)

        hot_valid = a1_valid.concat(a0_valid)
        # grant is a one-hot vector
        grant_1h = RegArray(Bits(2), 1, initializer=[1])
        gv = grant_1h[0]
        gv_flip = ~gv
        hi = gv_flip & hot_valid
        lo = gv & hot_valid
        hi_nez = ~(hi == Bits(2)(0))
        new_grant = hi_nez.select(hi, lo)
        grant0 = new_grant == Bits(2)(1)
        grant1 = new_grant == Bits(2)(2)
        with Condition(grant0):
            log("grants even")
            a0 = self.a0.pop()
            sqr.async_called(a = a0)
            (grant_1h & self)[0] <= Bits(2)(1)
        with Condition(grant1):
            log("grants odd")
            a1 = self.a1.pop()
            sqr.async_called(a = a1)
            (grant_1h & self)[0] <= Bits(2)(2)

class Driver(Module):

        def __init__(self):
            super().__init__(ports={})

        @module.combinational
        def build(self, arb: Arbiter):
            cnt = RegArray(Int(32), 1)
            k = cnt[0]
            v = k + Int(32)(1)
            even = v * Int(32)(2)
            even = even[0: 31]
            even = even.bitcast(Int(32))
            odd = even + Int(32)(1)
            (cnt & self)[0] <= v
            is_odd = v[0: 0]
            with Condition(is_odd):
                # arb.async_called(a0 = even, a1 = odd)
                arb.async_called(a0 = even)
                arb.async_called(a1 = odd)

def build_system():
    sqr = Squarer()
    sqr.build()

    arb = Arbiter()
    arb.build(sqr)

    driver = Driver()
    driver.build(arb)

def check(raw):
    last_grant = None
    for i in raw.split('\n'):
        if "grants odd" in i:
            assert (last_grant is None or last_grant == 0)
            last_grant = 1
        if "grants even" in i:
            assert (last_grant is None or last_grant == 1)
            last_grant = 0


def test_arbiter():
    run_test('arbiter', build_system, check, sim_threshold=200, idle_threshold=200)

if __name__ == '__main__':
    test_arbiter()
