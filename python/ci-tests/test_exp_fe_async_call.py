from assassyn.experimental.frontend import if_
from assassyn.experimental.frontend.factory import Factory, factory
from assassyn.experimental.frontend.module import pop_all
from assassyn.frontend import Module, RegArray, Port, UInt, log
from assassyn.test import run_test

@factory(Module)
def adder_factory() -> Factory[Module]:
    def adder(a: Port[UInt(32)], b: Port[UInt(32)]):
        a, b = pop_all(True)
        c = a + b
        log("Adder: {} + {} = {}", a, b, c)
    return adder

@factory(Module)
def driver_factory(adder: Factory[Module]) -> Factory[Module]:
    def driver():
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        with if_(cnt[0] < UInt(32)(100)):
            (adder << (cnt[0], cnt[0]))()
    return driver

def top():
    adder = adder_factory()
    driver_factory(adder)

def check_raw(raw):
    cnt = 0
    # print(raw)
    for i in raw.split('\n'):
        if 'Adder:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            # print(a,b,c)
            assert int(a) + int(b) == int(c)
            cnt += 1
    # print(cnt)
    assert cnt == 100, f'cnt: {cnt} != 100'

def test_exp_fe_async_call():
    run_test('exp_fe_async_call', top, check_raw,
             sim_threshold=200, idle_threshold=200, random=True)


if __name__ == '__main__':
    test_exp_fe_async_call()
