import pytest

from assassyn.experimental.frontend.factory import factory, this
from assassyn.experimental.frontend.module import ModuleFactory, pop_all
from assassyn.frontend import Module, Port, RegArray, UInt, Bits, Int, log, wait_until, Condition
from assassyn.test import run_test


@factory(Module)
def squarer_factory() -> ModuleFactory:
    def squarer(a: Port[Int(32)]):
        a = pop_all(validate=False)
        b = a * a
        log("adder: {} * {} = {}", a, a, b)
    return squarer


@factory(Module)
def arbiter_factory(sqr: ModuleFactory) -> ModuleFactory:
    def arbiter(a0: Port[Int(32)], a1: Port[Int(32)]):
        a0_valid = a0.valid()
        a1_valid = a1.valid()
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
            a0_val = a0.pop()
            (sqr << {'a': a0_val})()
            (grant_1h & this())[0] <= Bits(2)(1)
        with Condition(grant1):
            log("grants odd")
            a1_val = a1.pop()
            (sqr << {'a': a1_val})()
            (grant_1h & this())[0] <= Bits(2)(2)
    return arbiter


@factory(Module)
def driver_factory(arb: ModuleFactory) -> ModuleFactory:
    def driver():
        cnt = RegArray(Int(32), 1)
        k = cnt[0]
        v = k + Int(32)(1)
        even = v * Int(32)(2)
        even = even[0: 31]
        even = even.bitcast(Int(32))
        odd = even + Int(32)(1)
        (cnt & this())[0] <= v
        is_odd = v[0: 0]
        with Condition(is_odd):
            # arb.async_called(a0 = even, a1 = odd)
            (arb << {'a0': even})()
            (arb << {'a1': odd})()
    return driver


def build_system():
    sqr = squarer_factory()
    arb = arbiter_factory(sqr)
    driver = driver_factory(arb)


def check(raw):
    last_grant = None
    for i in raw.split('\n'):
        if "grants odd" in i:
            assert (last_grant is None or last_grant == 0)
            last_grant = 1
        if "grants even" in i:
            assert (last_grant is None or last_grant == 1)
            last_grant = 0


def test_exp_fe_arbiter():
    run_test('arbiter_exp', build_system, check, sim_threshold=200, idle_threshold=200)


if __name__ == '__main__':
    test_exp_fe_arbiter()
