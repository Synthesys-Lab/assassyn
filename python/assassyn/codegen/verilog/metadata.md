# Verilog Metadata Package

The `python.assassyn.codegen.verilog.metadata` package owns the data structures
that capture how IR modules interact with arrays, FIFOs, and async callees
during Verilog code generation.  It folds the legacy monolithic
`metadata.py` file into focused submodules while preserving the public API
exposed through the package `__init__`.

- `metadata.core` – shared enums, the `InteractionMatrix`, and `AsyncLedger`.
- `metadata.module` – module-scoped helpers (`ModuleBundle`,
  `ModuleInteractionView`, and `ModuleMetadata`).
- `metadata.array` – array projections and the `ArrayMetadata` registry.
- `metadata.fifo` – FIFO-centric projections consumed by downstream emitters.

Existing call sites continue to import from
`python.assassyn.codegen.verilog.metadata` without code changes thanks to the
package re-exports.  Only the implementation layout shifts; behaviour and
interfaces remain the same.

## Summary

The metadata pre-pass records every interaction in a cross-product matrix
indexed by `(module, resource, role)`:

- **module** – the IR `Module` that emitted the expression.
- **resource** – either an IR `Array` or FIFO `Port`.
- **role** – `InteractionKind.ARRAY_READ`, `ARRAY_WRITE`, `FIFO_PUSH`,
  or `FIFO_POP`.

Each interaction is captured exactly once as an IR expression (carrying its
predicate via `expr.meta_cond`).  The matrix exposes immutable module- and
resource-scoped views after `freeze()` runs, so every downstream phase can query
the same snapshot without defensive copying.  An `AsyncLedger` complements the
matrix by grouping async calls by caller and callee, and `ModuleMetadata` keeps
the remaining module-scoped data (FINISH intrinsics, async call list, value
exposures) together with the module view returned by the matrix.

## Package Exports

```python
from python.assassyn.codegen.verilog import metadata

metadata.InteractionKind
metadata.InteractionMatrix
metadata.FIFOExpr
metadata.ModuleMetadata
metadata.AsyncLedger
metadata.ModuleBundle
metadata.ModuleInteractionView
metadata.ArrayInteractionView
metadata.FIFOInteractionView
metadata.ArrayMetadata
metadata.ExternalRegistry
metadata.ExternalRead
```

All public types are re-exported by the package root; consumers do not need to
change their import statements.  The following sections document the owning
submodule for each class.

## `metadata.core` – shared types

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
interactions, and downstream consumers rely on consistent naming when selecting
data from the projections.

### `FIFOExpr`

Alias for the union of `FIFOPush` and `FIFOPop`.  The shared type keeps FIFO
helpers consistent across the module and matrix implementations.

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
Until `freeze()` runs, buckets are append-only lists.  `freeze()` snaps those
lists to tuples, memoises the view adapters, and flips an internal flag that
prevents further mutation.  After freezing, `module_view`, `array_view`, and
`fifo_view` return cached adapters backed by the same immutable tuples—callers
never allocate new containers or duplicate expression references.

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
and by callee (for trigger aggregation).  All queries require the ledger to be
frozen; attempting to inspect an unfrozen ledger raises an error.  When
`freeze()` runs, lists are converted to tuples, lookup mappings become
`MappingProxyType` instances, and the ledger refuses further mutation.

## `metadata.module` – module-scoped helpers

```python
@dataclass
class ModuleMetadata:
    module: Module
    matrix: InteractionMatrix
    ...
```

- `ModuleBundle` accumulates mutable buckets per module while the matrix remains
  unfrozen.
- `ModuleInteractionView` is an immutable named tuple exposing FIFO pushes/pops,
  FIFO maps, and array read/write groupings scoped to the module.
- `ModuleMetadata` packages module-scoped metadata (value exposures, FINISH
  intrinsics, async calls) alongside the module view obtained from the matrix.
  Callers must invoke `ModuleMetadata.freeze()` before inspecting
  `interactions`, `finish_sites`, or `calls`; attempting to read them prior to
  freezing raises an exception.

## `metadata.array` – array projections

```python
@dataclass
class ArrayMetadata:
    array: Array
    write_ports: dict[Module, int]
    read_ports_by_module: dict[Module, list[int]]
    read_order: list[tuple[Module, ArrayRead]]
    read_expr_port: dict[ArrayRead, int]
    users: list[Module]
```

- `ArrayInteractionView` exposes immutable tuples of read expressions, per-writer
  mappings of array writes, and per-module read buckets.
- `ArrayMetadataRegistry` (in `python/assassyn/codegen/verilog/array.py`)
  consumes this data to assign port indices; rebuilds pull from the shared view
  to keep numbering consistent with the traversal order captured by the matrix.

## `metadata.fifo` – FIFO projections

```python
class FIFOInteractionView(NamedTuple):
    pushes: tuple[FIFOPush, ...]
    pops: tuple[FIFOPop, ...]
```

The FIFO view provides the resource-level counterpart to the module view.
Downstream emitters fetch cross-module traffic without re-walking the IR by
consulting the matrix for a port and reading these tuples directly.

## `metadata.external` – external module metadata

```python
@dataclass
class ExternalRead:
    expr: PureIntrinsic
    producer: Module
    consumer: Module
    instance: ExternalIntrinsic
    port_name: str
    index_operand: Value | None

class ExternalRegistry:
    def record_instance(self, instance: ExternalIntrinsic, owner: Module) -> None: ...
    def record_cross_module_read(self, *, expr, producer, consumer, instance, port_name, index_operand) -> None: ...
    def freeze(self) -> None: ...
    def reads_for_consumer(self, module: Module) -> tuple[ExternalRead, ...]: ...
```

- `ExternalRegistry` stores the external classes encountered during analysis,
  the owning module for each `ExternalIntrinsic`, and every cross-module read of
  external outputs.  After `freeze()` the registry serves immutable views so
  module emitters, cleanup, and top-level wiring can reuse a single source of
  truth when synthesising exposed data/valid ports.

## `metadata.__init__` – public surface

The package entry point re-exports all the classes above along with the shared
`FIFOExpr` alias so existing imports (`from .metadata import ...`) continue to
resolve without modification.  Consumers can opt into submodule imports for
more granular dependencies when desired.

## Implementation Notes

- Submodules rely on `typing.TYPE_CHECKING` to avoid import cycles while keeping
  type hints precise.  Shared aliases live in `metadata.core`.
- `InteractionMatrix.freeze()` must run before any consumer inspects module or
  resource views.  Attempting to mutate the matrix (or module metadata) after
  freezing raises a `RuntimeError`.
- The package layout reduces the maintenance burden by assigning
  responsibilities to self-contained modules while retaining deterministic
  ordering guarantees required by Verilog emission.
