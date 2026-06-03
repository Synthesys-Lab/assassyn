# Simulator Versus Verilator Benchmark Smoke Test

## Summary

`test_sim_vs_verilator_benchmark.py` verifies benchmark metric collection without
running a real Cargo or Verilator build. The test monkeypatches Assassyn backend
entry points so it can assert configuration and report structure deterministically.

## Exposed Interfaces

This file exposes pytest tests only.

## Internal Helpers

- `BenchSink`: one-port consumer for the smoke-test design.
- `BenchDriver`: producer that sends one async call to `BenchSink`.
- `build_system`: creates the deterministic `SysBuilder`.
- `fake_elaborate`: local monkeypatched backend replacement inside the test.

## Data Structures

The test exercises `BenchmarkConfig`, `BenchmarkMetric`, and `BenchmarkReport`
from `assassyn.benchmark.sim_vs_verilator`.
