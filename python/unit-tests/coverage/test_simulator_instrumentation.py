"""Tests for simulator coverage configuration and generated instrumentation."""

from pathlib import Path

from assassyn import backend
from assassyn.frontend import Int, Module, Port, SysBuilder, log, module


class Sink(Module):
    """Small consumer used to force FIFO and module coverage points."""

    def __init__(self):
        super().__init__(ports={"data": Port(Int(32))})

    @module.combinational
    def build(self):
        data = self.pop_all_ports(True)
        log("sink {}", data)


class Driver(Module):
    """Small producer used to trigger semantic coverage instrumentation."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, sink: Sink):
        sink.async_called(data=Int(32)(7))


def build_system():
    """Create a minimal async-call system."""

    sysb = SysBuilder("coverage_instrumentation")
    with sysb:
        sink = Sink()
        sink.build()
        driver = Driver()
        driver.build(sink)
    return sysb


def test_backend_config_accepts_coverage_options():
    """Coverage configuration is accepted by the public backend config."""

    cfg = backend.config(
        coverage=True,
        coverage_path="coverage.json",
        coverage_roi=(2, 9),
    )

    assert cfg["coverage"] is True
    assert cfg["coverage_path"] == "coverage.json"
    assert cfg["coverage_roi"] == (2, 9)


def test_generated_simulator_contains_coverage_hooks(tmp_path):
    """Generated Rust simulator contains recorder construction and FIFO probes."""

    sysb = build_system()
    backend.elaborate(
        sysb,
        path=str(tmp_path),
        simulator=True,
        verilog=False,
        verbose=False,
        enable_cache=False,
        coverage=True,
        coverage_path="coverage.json",
        coverage_roi=(0, 4),
        sim_threshold=4,
    )

    simulator_rs = next(Path(tmp_path).glob("**/src/simulator.rs"))
    generated = simulator_rs.read_text(encoding="utf-8")

    assert "CoverageRecorder" in generated
    assert "coverage.record_module" in generated
    assert "coverage.record_fifo_push" in generated
    assert "coverage.flush" in generated
