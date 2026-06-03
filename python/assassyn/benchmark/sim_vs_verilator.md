# Rust Simulator Versus Verilator Benchmark

## Summary

`sim_vs_verilator.py` measures the same Assassyn system through three execution
paths:

- generated Rust simulator;
- generated Rust simulator with semantic coverage enabled;
- generated RTL through the existing Verilator/cocotb path when available.

The harness reports wall-clock elaboration time, run time, total time, output
size, and ROI cycles. It intentionally avoids machine-specific pass/fail
thresholds.

## Exposed Interfaces

### `BenchmarkConfig`

Configures the output path, simulation threshold, idle threshold, coverage ROI,
Verilator enable policy, and build-cache policy.

### `BenchmarkMetric`

Stores one backend measurement and exposes `total_seconds` plus
`to_json_dict()`.

### `BenchmarkReport`

Stores all measurements for one design and exposes `to_json_dict()`.

### `run_sim_vs_verilator`

```python
def run_sim_vs_verilator(sys, config: BenchmarkConfig) -> BenchmarkReport
```

Runs the Rust, Rust+coverage, and optional Verilator paths in separate output
directories under `config.path`.

### `render_markdown_report`

Converts a report into a compact Markdown table suitable for `.reports/`.

## Internal Helpers

- `_run_rust_simulator`: measures Rust simulator elaboration and execution.
- `_run_coverage_simulator`: measures Rust simulator execution with semantic
  coverage enabled and records the coverage JSON path.
- `_run_verilator`: measures Verilog elaboration and the existing Verilator
  testbench path.
- `_time_call`: wraps a function call with `perf_counter` timing.
- `_metric`: converts backend output into `BenchmarkMetric`.
- `_roi_cycle_count`: computes inclusive ROI cycle count.
- `_idle_threshold`: resolves default idle threshold.
- `_verilator_enabled`: resolves the Verilator availability policy.

## Data Structures

The report is intentionally plain dataclasses so CI tests and later report
scripts can serialize it without depending on pandas or benchmark-specific
packages.
