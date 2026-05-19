# Deterministic CSA Multiplier Test

This test covers the shared carry-save multiplier IP in
`assassyn.ip.multiply` using the Rust simulator only.

## Interface Exposed

- `test_csa_multiplier()`: Builds and runs the deterministic multiplier system
  with `run_test(..., verilog=False)`.
- `build_system()`: Instantiates `Driver`, which calls the shared `multiply`
  helper every cycle.
- `check_raw(raw)`: Parses `CsaMultiplierResult` logs and compares each product
  against Python's integer multiplication masked to 64 bits.

## Internal Helpers

- `_select_vector_value(selector, values)`: Builds an Assassyn `case` tree that
  selects fixed 32-bit operand constants from the deterministic vector list.
- `RESULT_RE`: Matches the multiplier stage-3 debug log containing tag,
  operands, and product.

## Data Structures

- `VECTORS`: Fixed unsigned 32-bit operand pairs. The set includes zero,
  identity, small dense values, high-bit operands, and the maximum
  `0xffffffff * 0xffffffff` case.
- `Driver`: A no-port Assassyn module with one `UInt(32)` cycle register. The
  cycle value selects operands, tags the pipeline transaction, and logs the
  externally visible registered multiplier result.
