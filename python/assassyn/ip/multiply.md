# Carry-Save Multiplier IP

`multiply.py` implements the shared Assassyn unsigned 32 by 32 multiplier IP.
The design replaces the previous bit-serial shift/add pipeline with a fixed
three-stage carry-save pipeline:

1. Generate and register 32 shifted partial-product rows.
2. Compress those rows through a 3:2 carry-save tree and register the two
   remaining rows.
3. Add the sum and carry rows, then register the 64-bit product.

The implementation intentionally does not use the Assassyn multiply operator
inside the IP. Radix-4 Booth recoding is not part of this replacement; it is a
future optimization after the CSA baseline is measured.

## Interface Exposed

### `multiply(a, b, cnt, debug=False)`

Builds one instance of the multiplier pipeline and asynchronously calls each
stage from the current module.

- `a`: 32-bit operand. It is bitcast to `UInt(32)` before entering the IP.
- `b`: 32-bit operand. It is bitcast to `UInt(32)` before entering the IP.
- `cnt`: 32-bit tag passed through the pipeline for debug correlation.
- `debug`: Python elaboration-time flag that enables stage logs.
- Returns: the registered `UInt(64)` product from the final stage.

The returned value is the pipeline output register. A caller that changes
operands every cycle must account for the stage latency in its own control
logic.

## Internal Helpers

### `_as_uint(value, width)`

Views an Assassyn value as `UInt(width)` using a bitcast when needed. This keeps
the public helper tolerant of signed or raw-bit callers while the IP internals
remain unsigned.

### `shifted_partial_product(a, b, bit_index)`

Builds one 64-bit partial-product row by selecting bit `bit_index` from operand
`b`, zero-extending operand `a`, shifting it by the selected weight, and using a
`select` to choose either the shifted row or zero.

### `carry_save_add(lhs, rhs, third)`

Implements one product-width 3:2 compressor:

- sum row: bitwise xor of the three inputs
- carry row: bitwise majority of the three inputs shifted left by one

Both outputs are returned as `UInt(64)`.

### `carry_save_reduce(rows)`

Builds a static compressor tree over the supplied product rows. Each Python
elaboration pass groups rows in threes, emits one `carry_save_add`, and carries
one or two leftover rows forward until only two rows remain.

## Pipeline Stages

### `PartialProductStage`

Ports:

- `a: UInt(32)`
- `b: UInt(32)`
- `tag: UInt(32)`

Writes:

- `partial_products`: 32 one-entry `RegArray(UInt(64), 1)` rows
- `stage1_a`, `stage1_b`, `stage1_tag`: one-entry metadata registers

### `CarrySaveStage`

Ports:

- `valid: UInt(1)`

Reads the stage-1 partial products and metadata registers, compresses the
partial products to two rows, and writes:

- `stage2_sum: RegArray(UInt(64), 1)`
- `stage2_carry: RegArray(UInt(64), 1)`
- `stage2_a`, `stage2_b`, `stage2_tag`: one-entry metadata registers

### `FinalAddStage`

Ports:

- `valid: UInt(1)`

Reads the stage-2 sum and carry rows, computes the final carry-propagate sum,
and writes:

- `result: RegArray(UInt(64), 1)`

With `debug=True`, this stage logs `CsaMultiplierResult` with the pipelined tag,
operands, and product.

## Data Structures

- `PRODUCT_WIDTH = 64`: product and CSA row width.
- `OPERAND_WIDTH = 32`: input operand and debug tag width.
- `PARTIAL_PRODUCT_COUNT = 32`: number of generated unsigned partial-product
  rows.
- Stage boundary arrays: the IP uses register arrays for all data moving
  between stages, while `async_called` stage invocations provide the FIFO
  trigger boundaries required by Assassyn's module model. The partial-product
  boundary uses one register array per row so the generator emits parallel
  single-write registers instead of multiple writes through one array port.
