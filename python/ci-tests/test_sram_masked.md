# Masked SRAM CI Test

`test_sram_masked.py` verifies that `SRAM.build(..., wmask=...)` preserves
masked-out bits and updates masked-in bits in the Rust simulator.

## Interface Exposed

- `test_sram_masked_write()`: Builds the masked SRAM regression system and runs
  it with `run_test`.
- `check(raw)`: Parses simulator logs from `ReadObserver` and compares them
  with the expected read table.

## Internal Helpers

- `ReadObserver`: Logs read responses with the step index and address.
- `Launcher`: Starts the async driver pipeline.
- `MaskedDriver`: Replays the static transaction table one operation per cycle,
  using Assassyn `select` and `Condition` for cycle-dependent hardware control.
- `top()`: Local builder inside the test that wires the driver, SRAM, launcher,
  and observer.

## Data Structures

- `MaskedOp`: Frozen dataclass describing one static read or write transaction.
  Write rows use `wdata` and `wmask`; read rows use `expected`.
- `OPS`: Ordered regression table covering full-word writes, zero masks, each
  byte lane, halfword masks, a non-byte-aligned bit mask, and cross-address
  isolation.
- `READ_RE`: Regex used to extract read log fields from simulator output.
