# DONE: Assassyn Translation Validation

Plan: `.todo/assassyn-translation-validation.md`

## Summary

Implemented the first translation-validation path for Assassyn:

- source-level semantic coverage in the generated Rust simulator;
- validation model extraction from Verilog metadata;
- shared Verilog schedule equations;
- validation JSON and bind-based SystemVerilog monitor artifacts;
- ROI-scoped Rust simulator versus Verilator benchmark harness.

## Key Artifacts

- `docs/design/internal/simulation-coverage.md`
- `docs/design/internal/translation-validation.md`
- `docs/design/internal/translation-validation-bug-study.md`
- `python/assassyn/verification/`
- `python/assassyn/benchmark/`
- `tools/rust-sim-runtime/src/runtime/coverage.rs`

## Verification

Focused tests for coverage, verification, benchmark smoke, schedule helpers, and
non-Verilator codegen paths passed. Real Verilator/cocotb execution is blocked
by missing `cocotb` in this environment.

## Follow-Up

Next work should add predicate-equation checking, array/register commit-boundary
checks, counterexample replay, and real Verilator performance measurements after
the environment is fixed.
