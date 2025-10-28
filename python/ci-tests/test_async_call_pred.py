from assassyn.frontend import *
from assassyn.test import run_test


class Adder(Module):

    def __init__(self):
        super().__init__(
            ports={
                "a": Port(Int(32)),
                "b": Port(Int(32)),
            },
        )

    @module.combinational
    def build(self):
        a, b = self.pop_all_ports(True)
        c = a + b
        log("Adder: {} + {} = {}", a, b, c)


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, adder: Adder):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)
        cond = cnt[0] < Int(32)(100)
        push_condition(cond)
        adder.async_called(a=cnt[0], b=cnt[0])
        pop_condition()


def check_raw(raw):
    cnt = 0
    for i in raw.split("\n"):
        if "Adder:" in i:
            line_toks = i.split()
            c = line_toks[-1]
            a = line_toks[-3]
            b = line_toks[-5]
            assert int(a) + int(b) == int(c)
            cnt += 1
    assert cnt == 100, f"cnt: {cnt} != 100"


def test_async_call_pred():
    def top():
        adder = Adder()
        adder.build()

        driver = Driver()
        driver.build(adder)

    run_test(
        "async_call_pred", top, check_raw, sim_threshold=200, idle_threshold=200, random=True, verilog=True
    )


if __name__ == "__main__":
    test_async_call_pred()

