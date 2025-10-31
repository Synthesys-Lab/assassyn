# Verilog Code Generation Metadata

This module owns the data structures that capture how IR modules interact with
arrays, FIFOs, and async callees during Verilog code generation.  The previous
design relied on separate containers (`ModuleExposure`, `ModuleFIFOView`,
`FIFORegistry`) that stored parallel views of the same expressions and guarded
each list with bespoke `_ensure_mutable` helpers.  The refactor replaces those
containers with a single `InteractionMatrix` plus lightweight view adapters,
eliminating redundant bookkeeping while keeping downstream emitters on fully
frozen tuples.

## Summary

The metadata pre-pass now records every interaction in a cross-product matrix
indexed by `(module, resource, role)`:

- **module** – the IR `Module` that emitted the expression.
- **resource** – either an IR `Array` or FIFO `Port`.
- **role** – `InteractionKind.ARRAY_READ`, `ARRAY_WRITE`, `FIFO_PUSH`,
  or `FIFO_POP`.

Each interaction is captured exactly once as an `InteractionRecord` that carries
the raw IR expression (and therefore its predicate via `expr.meta_cond`).  The
matrix exposes immutable module- and resource-scoped views after `freeze()`
runs, so every downstream phase can query the same snapshot without defensive
copying.  An `AsyncLedger` complements the matrix by grouping async calls by
caller and callee, and `ModuleMetadata` keeps the remaining module-scoped data
(FINISH intrinsics, async call list, value exposures) together with the module
view returned by the matrix.

## Exposed Interfaces

### `InteractionKind`

```python
class InteractionKind(Enum):
    ARRAY_READ = auto()
    ARRAY_WRITE = auto()
    FIFO_PUSH = auto()
    FIFO_POP = auto()
```

The enum labels the role an expression plays relative to a resource.  It
provides a stable set of keys the matrix uses internally when recording
interactions and its consumers rely on consistent naming when selecting data
from the projections.

### `InteractionRecord`

```python
@dataclass(frozen=True)
class InteractionRecord:
    module: Module
    resource: Array | Port
    kind: InteractionKind
    expr: Expr
```

Each record is created by the analysis pass and stored inside the matrix.  The
record itself is immutable; view adapters expose only the `expr` handles so
callers interact with familiar IR nodes.

### `InteractionMatrix`

```python
class InteractionMatrix:
    def record(self, *, module, resource, kind, expr) -> None: ...
    def module_view(self, module: Module) -> ModuleInteractionView: ...
    def array_view(self, array: Array) -> ArrayInteractionView: ...
    def fifo_view(self, port: Port) -> FIFOInteractionView: ...
    def freeze(self) -> None: ...
```

The matrix is the central accumulator.  During analysis the visitor calls
`record()` for every interaction; the matrix stores the interaction once and
updates the necessary module/resource buckets so both projections stay in sync.
Until `freeze()` runs, buckets are simple append-only lists.  `freeze()` snaps
those lists to tuples, memoises the view adapters, and flips an internal flag
that prevents further mutation.  After freezing, `module_view`, `array_view`,
and `fifo_view` return cached adapters backed by the same immutable tuples—
callers never allocate new containers or duplicate expression references.

The matrix also exposes the shared `async_ledger` attribute and keeps track of
`finish_sites` registered for each module so `ModuleMetadata` can surface the
same tuple the visitor collected.

### `ModuleInteractionView`

`ModuleInteractionView` is a lightweight named tuple with attributes:

- `pushes` / `pops` – tuples of FIFO expressions emitted by the module.
- `fifo_ports` – FIFO ports touched by the module in encounter order.
- `fifo_map` – mapping from FIFO port to the ordered tuple of interactions the
  module emitted for that port.
- `writes` / `reads` – mappings from arrays to tuples of write/read expressions
  produced by the module.
- `matrix` – back-reference to the owning `InteractionMatrix` so helpers can
  look up resource-scoped projections.

Code that previously asked for `resources(kind)` now inspects
`module_view.writes.keys()`, `module_view.reads.keys()`, or `module_view.fifo_ports`
directly.  Iterating FIFO channels simply walks `module_view.fifo_ports` and
consults both the module map and `matrix.fifo_view(port)`.

### `ArrayInteractionView`

The array projection is a named tuple with attributes `reads`, `writers`, and
`reads_by_module`.  Each attribute is a tuple or mapping of tuples, all frozen
after `freeze()`.  Consumers such as `ArrayMetadataRegistry` reuse these results
when assigning port indices, guaranteeing that numbering reflects the same
encounter order reported by the module view.

### `FIFOInteractionView`

`FIFOInteractionView` is a named tuple exposing `pushes` and `pops` tuples.  It
serves as the resource-level counterpart to the module view.  Downstream
emitters fetch cross-module traffic without re-walking the IR by consulting the
matrix for a port and reading these tuples directly.

### `AsyncLedger`

```python
class AsyncLedger:
    def record(self, module: Module, callee: Module, call: AsyncCall) -> None: ...
    def calls_for_module(self, module: Module) -> Mapping[Module, tuple[AsyncCall, ...]]: ...
    def calls_by_callee(self, callee: Module) -> tuple[AsyncCall, ...]: ...
    def freeze(self) -> None: ...
```

Async calls are tracked separately from array/FIFO interactions so trigger
accounting remains explicit.  The ledger groups calls by caller (for cleanup)
and by callee (for trigger aggregation analytics).  All queries require the
ledger to be frozen; attempting to inspect an unfrozen ledger raises an error.
When `freeze()` runs, each list is converted to a tuple, lookup mappings become
`MappingProxyType` instances, and the ledger refuses further mutation.

### `ModuleMetadata`

```python
@dataclass
class ModuleMetadata:
    module: Module
    interactions: ModuleInteractionView
    value_exposures: tuple[Expr, ...]
    finish_sites: tuple[Intrinsic, ...]
    calls: tuple[AsyncCall, ...]
```

`ModuleMetadata` packages module-scoped metadata that is not already embedded in
the matrix.  The analysis visitor appends value exposures, FINISH intrinsics,
and async calls to mutable lists while visiting the module; when the snapshot
is finalised it asks the matrix for the module view, freezes the ledger, and
converts the lists to tuples.  Downstream code therefore consumes a read-only
object:

- `interactions` gives access to arrays and FIFOs through the unified API.
- `value_exposures` replaces `ModuleExposure.values`.
- `finish_sites` carries the recorded FINISH intrinsics.
- `calls` mirrors the async call list used for wiring trigger counters.

The class no longer owns a registry pointer or bespoke `_ensure_mutable`
helpers—the matrix guarantees immutability once `freeze()` completes.

## Internal Helpers and Freeze Semantics

The module defines small helper classes (`_ModuleBuckets`, `_ArrayBuckets`,
`_FIFOBuckets`) that accumulate interactions while the matrix is mutable.  They
store plain Python lists and `OrderedDict` instances so encounter order is
naturally preserved.  `InteractionMatrix.freeze()` walks each bucket, converts
lists to tuples, and replaces mutable dictionaries with read-only mappings.
Attempting to call `record()` after freezing raises `RuntimeError`, preventing
accidental post-pass mutation.

The refactor reduces `metadata.py` by more than 100 lines by removing the
legacy `ModuleExposure`, `FIFOMetadata`, and `_ensure_mutable` boilerplate while
retaining the deterministic ordering guarantees required by Verilog emission.
