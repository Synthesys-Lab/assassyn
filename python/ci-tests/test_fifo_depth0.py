from assassyn.frontend import *
from assassyn.test import run_test


class Sink(Module):

    def __init__(self):
        super().__init__(
            ports={
                "x": Port(Int(32)),
            },
        )

    @module.combinational
    def build(self):
        x = self.pop_all_ports(False)
        log("Sink: {}", x)

        seen = RegArray(Int(32), 1)
        next_seen = seen[0] + Int(32)(1)
        (seen & self)[0] <= next_seen
        with Condition(next_seen >= Int(32)(3)):
            finish()


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, sink: Sink):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)

        with Condition(cnt[0] < Int(32)(3)):
            call = sink.async_called(x=cnt[0])
            call.bind.set_fifo_depth(x=0)


def check_raw(raw: str) -> None:
    values = []
    for line in raw.split("\n"):
        if "Sink:" not in line:
            continue
        values.append(int(line.split()[-1]))
    assert values == [0, 1, 2]


def test_fifo_depth0():
    def top():
        sink = Sink()
        sink.build()

        driver = Driver()
        driver.build(sink)

    run_test(
        "fifo_depth0",
        top,
        check_raw,
        fifo_depth=0,
        sim_threshold=50,
        idle_threshold=50,
    )


if __name__ == "__main__":
    test_fifo_depth0()
