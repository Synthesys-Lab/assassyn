# Benchmark Package Exports

## Summary

`__init__.py` exposes the public benchmark helpers without importing generated
designs or running toolchains.

## Exposed Interfaces

- `BenchmarkConfig`: benchmark configuration dataclass.
- `BenchmarkMetric`: per-backend timing and metadata dataclass.
- `BenchmarkReport`: benchmark result container.
- `run_sim_vs_verilator`: run Rust simulator, Rust simulator with coverage, and
  optional Verilator comparisons.
- `render_markdown_report`: convert a report to Markdown.

## Internal Helpers

This module only re-exports helpers from `sim_vs_verilator.py`.
