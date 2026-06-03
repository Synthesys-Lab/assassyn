"""ROI-scoped benchmark harness for Rust simulation and Verilator."""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from time import perf_counter
from typing import Any, Callable

from assassyn import backend, utils


@dataclass(frozen=True)
class BenchmarkConfig:
    """Configuration for one simulator-versus-Verilator benchmark."""

    path: str | Path
    sim_threshold: int = 64
    idle_threshold: int | None = None
    coverage_roi: tuple[int, int] | None = None
    enable_verilator: bool | None = None
    enable_cache: bool = False


@dataclass(frozen=True)
class BenchmarkMetric:  # pylint: disable=too-many-instance-attributes
    """Timing and metadata for one backend run."""

    backend: str
    available: bool
    elaborate_seconds: float
    run_seconds: float
    roi_cycles: int
    output_bytes: int
    coverage_path: str | None = None
    note: str | None = None

    @property
    def total_seconds(self) -> float:
        """Return elaboration plus execution wall time."""

        return self.elaborate_seconds + self.run_seconds

    def to_json_dict(self) -> dict[str, Any]:
        """Return a JSON-serializable metric dictionary."""

        return {
            "backend": self.backend,
            "available": self.available,
            "elaborate_seconds": self.elaborate_seconds,
            "run_seconds": self.run_seconds,
            "total_seconds": self.total_seconds,
            "roi_cycles": self.roi_cycles,
            "output_bytes": self.output_bytes,
            "coverage_path": self.coverage_path,
            "note": self.note,
        }


@dataclass(frozen=True)
class BenchmarkReport:
    """Result of one Assassyn simulator-versus-Verilator comparison."""

    design_name: str
    sim_threshold: int
    coverage_roi: tuple[int, int] | None
    metrics: tuple[BenchmarkMetric, ...] = field(default_factory=tuple)

    def to_json_dict(self) -> dict[str, Any]:
        """Return a JSON-serializable benchmark report."""

        return {
            "design_name": self.design_name,
            "sim_threshold": self.sim_threshold,
            "coverage_roi": self.coverage_roi,
            "metrics": [metric.to_json_dict() for metric in self.metrics],
        }


def run_sim_vs_verilator(sys, config: BenchmarkConfig) -> BenchmarkReport:
    """Run Rust simulation, coverage simulation, and optional Verilator."""

    base_path = Path(config.path)
    base_path.mkdir(parents=True, exist_ok=True)
    roi_cycles = _roi_cycle_count(config.coverage_roi, config.sim_threshold)

    metrics = [
        _run_rust_simulator(sys, config, base_path / "rust", roi_cycles),
        _run_coverage_simulator(sys, config, base_path / "rust_coverage", roi_cycles),
    ]
    metrics.append(_run_verilator(sys, config, base_path / "verilator", roi_cycles))

    return BenchmarkReport(
        design_name=getattr(sys, "name", "unknown"),
        sim_threshold=config.sim_threshold,
        coverage_roi=config.coverage_roi,
        metrics=tuple(metrics),
    )


def render_markdown_report(report: BenchmarkReport) -> str:
    """Render a compact Markdown table for benchmark reports."""

    lines = [
        f"# Assassyn Simulation Benchmark: {report.design_name}",
        "",
        f"- `sim_threshold`: {report.sim_threshold}",
        f"- `coverage_roi`: {report.coverage_roi}",
        "",
        "| Backend | Available | Elaborate s | Run s | Total s | "
        "ROI cycles | Output bytes | Note |",
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |",
    ]
    for metric in report.metrics:
        lines.append(
            f"| {metric.backend} | {metric.available} | "
            f"{metric.elaborate_seconds:.6f} | {metric.run_seconds:.6f} | "
            f"{metric.total_seconds:.6f} | {metric.roi_cycles} | "
            f"{metric.output_bytes} | {metric.note or ''} |"
        )
    lines.extend([
        "",
        "Verilator run time is reported as the generated `tb.py` wall time, which "
        "currently includes cocotb/Verilator build and execution work.",
    ])
    return "\n".join(lines) + "\n"


def _run_rust_simulator(
    sys,
    config: BenchmarkConfig,
    path: Path,
    roi_cycles: int,
) -> BenchmarkMetric:
    """Measure generated Rust simulator elaboration and execution."""

    (simulator_path, _), elaborate_seconds = _time_call(
        lambda: backend.elaborate(
            sys,
            path=str(path),
            simulator=True,
            verilog=False,
            verbose=False,
            enable_cache=config.enable_cache,
            sim_threshold=config.sim_threshold,
            idle_threshold=_idle_threshold(config),
        )
    )
    output, run_seconds = _time_call(lambda: utils.run_simulator(simulator_path))
    return _metric("rust", (elaborate_seconds, run_seconds), roi_cycles, output)


def _run_coverage_simulator(
    sys,
    config: BenchmarkConfig,
    path: Path,
    roi_cycles: int,
) -> BenchmarkMetric:
    """Measure generated Rust simulator execution with semantic coverage."""

    coverage_path = path / "coverage.json"
    (simulator_path, _), elaborate_seconds = _time_call(
        lambda: backend.elaborate(
            sys,
            path=str(path),
            simulator=True,
            verilog=False,
            verbose=False,
            enable_cache=config.enable_cache,
            coverage=True,
            coverage_path=str(coverage_path),
            coverage_roi=config.coverage_roi,
            sim_threshold=config.sim_threshold,
            idle_threshold=_idle_threshold(config),
        )
    )
    output, run_seconds = _time_call(lambda: utils.run_simulator(simulator_path))
    return _metric(
        "rust_coverage",
        (elaborate_seconds, run_seconds),
        roi_cycles,
        output,
        coverage_path=str(coverage_path),
    )


def _run_verilator(
    sys,
    config: BenchmarkConfig,
    path: Path,
    roi_cycles: int,
) -> BenchmarkMetric:
    """Measure generated RTL execution through the Verilator path."""

    if not _verilator_enabled(config):
        return BenchmarkMetric(
            backend="verilator",
            available=False,
            elaborate_seconds=0.0,
            run_seconds=0.0,
            roi_cycles=roi_cycles,
            output_bytes=0,
            note="Verilator unavailable or disabled",
        )

    (_, verilator_path), elaborate_seconds = _time_call(
        lambda: backend.elaborate(
            sys,
            path=str(path),
            simulator=False,
            verilog=True,
            verbose=False,
            enable_cache=config.enable_cache,
            sim_threshold=config.sim_threshold,
            idle_threshold=_idle_threshold(config),
        )
    )
    output, run_seconds = _time_call(lambda: utils.run_verilator(verilator_path))
    return _metric("verilator", (elaborate_seconds, run_seconds), roi_cycles, output)


def _time_call(callback: Callable[[], Any]) -> tuple[Any, float]:
    """Return a function result and elapsed wall time."""

    start = perf_counter()
    result = callback()
    return result, perf_counter() - start


def _metric(
    backend_name: str,
    timings: tuple[float, float],
    roi_cycles: int,
    output: str,
    coverage_path: str | None = None,
) -> BenchmarkMetric:
    """Create a benchmark metric from raw output."""

    elaborate_seconds, run_seconds = timings
    return BenchmarkMetric(
        backend=backend_name,
        available=True,
        elaborate_seconds=elaborate_seconds,
        run_seconds=run_seconds,
        roi_cycles=roi_cycles,
        output_bytes=len(output.encode("utf-8")),
        coverage_path=coverage_path,
    )


def _roi_cycle_count(roi: tuple[int, int] | None, sim_threshold: int) -> int:
    """Return the number of cycles in the configured ROI."""

    if roi is None:
        return sim_threshold
    start, end = roi
    if end < start:
        return 0
    return end - start + 1


def _idle_threshold(config: BenchmarkConfig) -> int:
    """Return the configured idle threshold."""

    if config.idle_threshold is not None:
        return config.idle_threshold
    return config.sim_threshold


def _verilator_enabled(config: BenchmarkConfig) -> bool:
    """Return whether the benchmark should run the Verilator path."""

    if config.enable_verilator is not None:
        return config.enable_verilator
    return utils.has_verilator() is not None
