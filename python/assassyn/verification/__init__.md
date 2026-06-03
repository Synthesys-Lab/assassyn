# Verification Package Initializer

## Summary

`__init__.py` re-exports the stable verification helper interfaces without
importing simulator or Verilog elaboration modules.

## Exposed Interfaces

- `load_coverage`
- `summarize_fifo_occupancy`
- `validate_coverage_schema`
- `build_validation_model`
- `render_monitor`
- `write_monitor_stub`
- `write_validation_json`
- validation model dataclasses from `model.py`

## Internal Helpers

This module has no internal helpers.
