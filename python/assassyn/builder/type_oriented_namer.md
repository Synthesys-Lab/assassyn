### `TypeOrientedNamer`

Provides type-aware prefixes and uniqueness guarantees for IR value names.

#### Methods

##### `__init__(self)`
Initialises a `UniqueNameCache` plus lookup tables for binary opcodes, unary
opcodes, and well-known IR classes.

##### `get_prefix_for_type(self, node: Any) -> str`
Inspects the node to choose a descriptive prefix:
- Known IR classes (`ArrayRead`, `ArrayWrite`, `FIFOPop`, `Concat`, `Select`,
  `Select1Hot`, `Slice`, `Cast`, `WireAssign`, `WireRead`, etc.) map to fixed
  string tags.
- Binary opcodes `200–216` (e.g. `add`, `sub`, `mul`, `eq`, `neq`, `shl`,
  `shr`) and unary opcodes `100–101` (`neg`, `not_op`) map to operation names.
- Nodes exposing a `dtype` attribute map `UInt`/`Int`/`Bits` to `u{bits}`,
  `i{bits}`, and `b{bits}` respectively, while `Float` currently maps to `f32`.
- Everything else falls back to `"val"`.

##### `name_value(self, value: Any, hint: Optional[str] = None) -> str`
Generates a unique identifier:
1. Sanitises the hint (if present) by replacing spaces and hyphens with `_`.
2. Passes either the cleaned hint or the derived prefix to the cache so the
   same semantic base yields incrementing suffixes (`foo`, `foo_1`, ...).

##### `reset(self)`
Replaces the internal cache with a new `UniqueNameCache`, effectively starting
fresh naming sequences.
