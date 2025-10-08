### `TypeOrientedNamer`

Provides type-aware prefixes and uniqueness guarantees for IR value names.

#### Methods

##### `__init__(self)`
Initialises a `UniqueNameCache` and lookup tables for the supported opcode
prefixes (`_binary_ops`, `_unary_ops`) together with a small catalogue of
class-to-prefix mappings used for arrays, FIFOs, async calls, etc.

##### `get_prefix_for_type(self, node: Any) -> str`
Extracts a descriptive base name by checking, in order:
1. Whether the node looks like a module instance (`ModuleBase` in its MRO).
2. FIFO helpers (`FIFOPop`, `FIFOPush`) where the FIFO's own semantic name is
   re-used when available.
3. Intrinsics that expose an opcode/argument pair (`PureIntrinsic`).
4. Known IR classes mapped through `_class_prefixes` such as `ArrayRead`,
   `ArrayWrite`, `Array`, `Concat`, `Select`, `Select1Hot`, `Slice`, `Cast`,
   `Bind`, `AsyncCall`, and FIFO helpers.
5. Opcode tables for arithmetic/logic operations.
6. A `name` attribute on the node.
7. A final fallback of `"val"`.

All intermediate names are sanitised via `_sanitize` so the identifier is
ASCII-only and uses `_` as separator.

##### `name_value(self, value: Any, hint: Optional[str] = None) -> str`
Generates a unique identifier:
1. Sanitises any explicit hint and returns the cached unique variant when set.
2. Otherwise, derives the prefix with `get_prefix_for_type`.
3. Passes the base through `UniqueNameCache` so later calls receive numbered
   suffixes (`foo`, `foo_1`, ...).
