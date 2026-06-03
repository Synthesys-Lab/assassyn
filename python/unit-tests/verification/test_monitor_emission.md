# Monitor Emission Tests

## Summary

`test_monitor_emission.py` verifies the generated translation-validation monitor
without invoking Verilator. It constructs a small `ValidationModel` and checks
that monitor text contains concrete RTL signal paths, `bind Top`, and FIFO /
trigger safety assertions.

## Exposed Interfaces

This file exposes pytest tests only.

## Internal Helpers

- `build_model`: creates a single-trigger, single-FIFO validation model.

## Data Structures

The tests exercise `ValidationModel`, `ModuleTransition`, `TriggerTransition`,
`FIFOTransition`, `RTLSignalMap`, and `AsyncCallTransition`.
