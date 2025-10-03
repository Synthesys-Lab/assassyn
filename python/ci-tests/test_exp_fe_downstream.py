from assassyn.experimental.frontend import factory, module, pin, Downstream, Module, Factory, Value
from assassyn.frontend import Port, UInt, log, RegArray
from assassyn.test import run_test


@factory(Module)
def forward_data_factory() -> Factory[Module]:
    def forward_data(data: Port[UInt(32)]):
        data = module.pop_all(True)
        pin(data)
    return forward_data


@factory(Downstream)
def adder_factory(a: Value, b: Value) -> Factory[Downstream]:
    def adder():
        a_val = a.optional(UInt(32)(1))
        b_val = b.optional(UInt(32)(1))
        c = a_val + b_val
        log("downstream: {} + {} = {}", a_val, b_val, c)
    return adder


@factory(Module)
def driver_factory(lhs: Factory[Module], rhs: Factory[Module]) -> Factory[Module]:
    def driver():
        cnt = RegArray(UInt(32), 1)
        v = cnt[0]
        cnt[0] = cnt[0] + UInt(32)(1)
        (lhs << {'data': v})()
        (rhs << {'data': v})()
    return driver


def check_raw(raw):
    cnt = 0
    for i in raw.split('\n'):
        if 'downstream:' in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 99, f'cnt: {cnt} != 99'


def test_exp_fe_downstream():
    def top():
        lhs = forward_data_factory()
        rhs = forward_data_factory()
        # Driver pushes data to both lhs and rhs
        driver_factory(lhs, rhs)
        adder_factory(lhs.pins[0], rhs.pins[0])

    run_test('exp_fe_downstream', top, check_raw,
             sim_threshold=100, idle_threshold=100)


if __name__ == '__main__':
    test_exp_fe_downstream()
