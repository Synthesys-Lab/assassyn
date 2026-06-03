# Monitor Emission Tests

## Summary

`test_monitor_emission.py` verifies the generated translation-validation monitor
without invoking Verilator. It constructs a small `ValidationModel` and checks
that monitor text contains concrete RegArray RTL signal paths, `bind Top`, the
generic `bind fifo` safety monitor, RegArray safety assertions,
activation/failure counter reports, and the generic `bind trigger_counter`
trigger safety monitor.

## Exposed Interfaces

This file exposes pytest tests only.

## Internal Helpers

- `build_model`: creates a single-trigger, single-FIFO validation model.

## Data Structures

The tests exercise `ValidationModel`, `ModuleTransition`, `TriggerTransition`,
`FIFOTransition`, `RTLSignalMap`, `ArrayTransition`, `ArrayWritePortTransition`,
`ArrayReadPortTransition`, and `AsyncCallTransition`.
