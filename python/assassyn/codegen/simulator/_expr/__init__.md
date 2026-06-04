# Simulator Expression Codegen

`__init__.py` dispatches Assassyn IR expression nodes to simulator Rust codegen
helpers. It owns the common expression lowering for log, slice, concat, select,
select1hot, cast, and the fallback dispatch table for specialized expression
modules.

## Exposed Interfaces

### `codegen_slice`

Lowers a static bit slice. Slices fully inside a 64-bit scalar cast the source
to `u64` and build the mask numerically. Wider slices cast to `BigUint` and call
the runtime `biguint_mask(width)` helper instead of emitting a long
`BigUint::parse_bytes("111...", 2)` string in the generated hot path.

### `codegen_concat`

Lowers two-value concatenation. When the result fits in 64 bits, the generated
code uses a `u64` fast path and casts the final value to the destination Rust
type. Wider concatenations keep the `BigUint` path.

### `codegen_cast`

Lowers zext, bitcast, and sext through the shared
`ValueCastTo::<Target>::cast(&value)` runtime interface.

### `codegen_expr`

Dispatches exact expression node types first, then falls back to `isinstance`
matches for subclasses. Array writes are passed the current module name because
their lowering needs the writer port context.

## Internal Data

`_EXPR_CODEGEN_DISPATCH` maps IR expression classes to their lowering
functions. The table keeps expression dispatch centralized while arithmetic,
array, call, and intrinsic lowering live in focused sibling modules.
