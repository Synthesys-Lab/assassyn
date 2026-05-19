# Timing Analysis Unit Tests

`test_timing.py` covers the pre-synthesis critical-path analyzer added under
`assassyn.analysis.timing`.

## Interface Exposed

- `test_critical_paths_reports_fifo_source_to_fifo_sink()`: Verifies that FIFO
  metadata creates a source-to-sink timing path through a combinational add.
- `test_write_critical_paths_report_uses_shared_filename()`: Verifies report
  writing, schema fields, and the shared `CRITICAL_PATHS_REPORT` filename.

## Internal Helpers

- `_build_fifo_adder_system()`: Builds a minimal Assassyn system with one
  module that pops a FIFO input, increments the value, and pushes it to a FIFO
  output.

## Data Structures

The test defines no persistent data structures beyond the local `Pipe` module
class inside `_build_fifo_adder_system()`. The assertions inspect the
`CriticalPath`, `TimingNode`, and `TimingEdge` objects returned by
`critical_paths()`.
