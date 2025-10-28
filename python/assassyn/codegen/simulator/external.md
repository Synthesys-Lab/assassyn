# External Module Utilities

Helper functions in `external.py` keep the simulator generator in sync with
`ExternalSV` modules and the newer `ExternalIntrinsic` expressions. The module
focuses on two jobs:

1. Tracking which IR values must be cached on the Rust side so downstream code
   can observe them reliably.
2. Naming simulator fields for external handles so that code generation remains
   consistent across passes.

With the migration to `ExternalIntrinsic`, legacy wire-assignment nodes are no
longer produced, so the helpers concentrate solely on exposure analysis and
metadata collection.

## Section 0. Summary

During simulator generation we walk the elaborated IR to discover:

- Which expressions escape their defining module (`collect_module_value_exposures`)
- Which expressions must have validity bits cached per module (`gather_expr_validities`)
- Which `ExternalSV` declarations elaborate to real bodies versus stub shells
  (`has_module_body` / `is_stub_external`)
- Which `ExternalIntrinsic` instances appear in the design so that Rust fields
  can be allocated in advance (`collect_external_intrinsics`), along with the set
  of distinct `ExternalSV` classes they reference (`collect_external_classes`)

All of these helpers are intentionally lightweight wrappers over existing
analyses so other simulator passes can share a consistent view of the IR.

## Section 1. Exposed Interfaces

### `external_handle_field`

```python
def external_handle_field(module_name: str) -> str:
```

Returns the field name used on the simulator struct to store the FFI handle for
an `ExternalSV` module. The name is derived from `namify(module_name)` followed
by the `_ffi` suffix.

### `collect_module_value_exposures`

```python
def collect_module_value_exposures(module: Module) -> Set[Expr]:
```

Runs `expr_externally_used` over a module body and returns the expressions whose
results are consumed outside the defining module. These expressions are the
candidates that require caching and validity tracking during simulation.

### `gather_expr_validities`

```python
def gather_expr_validities(sys) -> Tuple[Set[Expr], Dict[Module, Set[Expr]]]:
```

Aggregates every expression that needs simulator-visible caching and produces
both a global set and a per-module map. The caller uses the result when
declaring `*_value` fields and validity bits on the simulator struct.

### `has_module_body` and `is_stub_external`

```python
def has_module_body(module: Module) -> bool:
def is_stub_external(module: Module) -> bool:
```

Helper predicates that distinguish fully elaborated modules from placeholder
stubs. They keep downstream passes from generating code for external modules
that have no synthesized body. The detection explicitly handles the new list
backed bodies produced by the builder as well as any legacy wrappers that still
expose a `.body` attribute.

### `collect_external_intrinsics`

```python
def collect_external_intrinsics(sys):
```

Walks the entire system and returns the `ExternalIntrinsic` instances that are
present. Simulator code uses this list to allocate per-instance FFI state.

### `collect_external_classes`

```python
def collect_external_classes(external_intrinsics) -> Dict[str, type]:
```

Post-processes the intrinsic list and collapses it to a mapping of unique
`ExternalSV` classes. This allows downstream generators to request one Verilator
crate per class even if multiple intrinsics reference the same external handle,
and keeps name allocation consistent for both module instances and intrinsic
users.

## Section 2. Internal Helpers

### `_ModuleValueExposureCollector`

A thin `Visitor` subclass that records expressions flagged by
`expr_externally_used`. The collector walks the flat module body list emitted by
the builder so it stays aligned with the block-free IR representation.

## Section 3. Design Notes

- Legacy helpers for `WireAssign` / `WireRead` were removed alongside the move
  to `ExternalIntrinsic`-based wiring. External connections are now handled by
  intrinsic code generation instead of bespoke visitors.
- The remaining helpers avoid unwrapping operands explicitly; `expr` objects are
  passed through as-is so other passes can decide how much information they
  need.
