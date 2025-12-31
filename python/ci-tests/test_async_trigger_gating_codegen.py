from pathlib import Path

from assassyn.frontend import *
from assassyn.backend import elaborate


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


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, sink: Sink):
        cnt = RegArray(Int(32), 1)
        (cnt & self)[0] <= cnt[0] + Int(32)(1)

        with Condition(cnt[0] < Int(32)(2)):
            call = sink.async_called(x=cnt[0])
            call.bind.set_fifo_depth(x=1)


def test_async_trigger_gating_codegen(tmp_path):
    sys = SysBuilder("async_trigger_gating_codegen")
    with sys:
        sink = Sink()
        sink.build()

        driver = Driver()
        driver.build(sink)

    _, verilog_path = elaborate(
        sys,
        path=str(tmp_path),
        verbose=False,
        simulator=False,
        verilog=True,
        enable_cache=False,
        sim_threshold=4,
        idle_threshold=4,
    )

    design_py = Path(verilog_path) / "design.py"
    design_text = design_py.read_text(encoding="utf-8").splitlines()

    trigger_assignments = []
    for idx, line in enumerate(design_text):
        if "Summing triggers for" not in line:
            continue
        for lookahead in range(idx, min(idx + 25, len(design_text))):
            candidate = design_text[lookahead].strip()
            if "_trigger =" in candidate:
                trigger_assignments.append(candidate)
                break

    assert trigger_assignments, "No async trigger assignments found in generated design.py"
    assert any("_push_ready" in line for line in trigger_assignments), (
        "Async trigger assignments are not gated by callee FIFO push_ready signals"
    )


if __name__ == "__main__":
    test_async_trigger_gating_codegen(Path("./workspace"))

