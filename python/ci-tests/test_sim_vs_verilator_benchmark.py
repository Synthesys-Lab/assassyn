"""Smoke tests for simulator-versus-Verilator benchmark metric collection."""

from pathlib import Path

from assassyn.benchmark import sim_vs_verilator as benchmark
from assassyn.frontend import Int, Module, Port, SysBuilder, finish, log, module


class BenchSink(Module):
    """Consumer used by the benchmark smoke system."""

    def __init__(self):
        super().__init__(ports={"data": Port(Int(32))})

    @module.combinational
    def build(self):
        """Consume one value and finish the smoke-test run."""

        data = self.pop_all_ports(True)
        log("bench sink {}", data)
        finish()


class BenchDriver(Module):
    """Producer used by the benchmark smoke system."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, sink: BenchSink):
        """Send one deterministic value to the sink."""

        sink.async_called(data=Int(32)(7))


def build_system():
    """Build a deterministic benchmark smoke-test system."""

    sysb = SysBuilder("sim_vs_verilator_benchmark")
    with sysb:
        sink = BenchSink()
        sink.build()
        driver = BenchDriver()
        driver.build(sink)
    return sysb


def test_sim_vs_verilator_benchmark_collects_metrics(monkeypatch, tmp_path):
    """Benchmark smoke test collects metrics without performance thresholds."""

    elaborate_calls = []

    def fake_elaborate(sys, **kwargs):
        del sys
        elaborate_calls.append(kwargs)
        output_dir = Path(kwargs["path"])
        output_dir.mkdir(parents=True, exist_ok=True)
        if kwargs.get("verilog"):
            return None, output_dir / "verilator"
        return output_dir / "Cargo.toml", None

    monkeypatch.setattr(benchmark.backend, "elaborate", fake_elaborate)
    monkeypatch.setattr(benchmark.utils, "run_simulator", lambda _path: "sim output")
    monkeypatch.setattr(benchmark.utils, "run_verilator", lambda _path: "rtl output")
    monkeypatch.setattr(benchmark.utils, "has_verilator", lambda: "verilator")

    config = benchmark.BenchmarkConfig(
        path=tmp_path,
        sim_threshold=8,
        coverage_roi=(2, 5),
        enable_verilator=True,
    )
    report = benchmark.run_sim_vs_verilator(build_system(), config)

    metrics = {metric.backend: metric for metric in report.metrics}
    assert set(metrics) == {"rust", "rust_coverage", "verilator"}
    assert metrics["rust"].available
    assert metrics["rust_coverage"].coverage_path is not None
    assert metrics["rust_coverage"].roi_cycles == 4
    assert metrics["verilator"].available
    assert any(call.get("coverage") for call in elaborate_calls)
    assert any(call.get("verilog") for call in elaborate_calls)
    assert "Verilator run time" in benchmark.render_markdown_report(report)
