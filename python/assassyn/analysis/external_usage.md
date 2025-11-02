# External Usage Analysis

This module centralises the bookkeeping needed to determine whether an `Expr`
escapes its defining module. The data is consumed by both the Verilog and
simulator back-ends to decide which values must be surfaced as exposed ports.

## Overview

1. **Preprocessing** – `build_external_usage_index` walks every module's
   `ModuleBase.externals` and records which modules consume a given expression.
   The resulting :class:`ExternalUsageIndex` is a lightweight, read-only
   registry that can be shared across multiple analysis passes.
2. **On-demand queries** – `ExternalUsageIndex.is_externally_used` answers
   cross-module usage checks in `O(1)` time by consulting the registry and, if
   needed, falling back to the expression's user list. The function caches the
   fallback result per `(expr, module)` to avoid repeated traversals.
3. **Legacy compatibility** – `expr_externally_used` retains its original
   signature and behaviour. When an index is supplied it delegates to
   `ExternalUsageIndex`, otherwise it performs the original user-list scan. This
   keeps existing callers working while enabling incremental adoption of the
   precomputed registry.

## Exposed Interfaces

### `get_module(operand: Operand) -> ModuleBase | None`

Returns the module that owns the provided operand by peeking at the operand's
user and its `parent` attribute. This helper underpins both the registry and the
legacy `expr_externally_used` function.

### `class ExternalUsageIndex`

```python
index = ExternalUsageIndex()
index.record_module_externals(module)
is_used = index.is_externally_used(expr, module)
```

Stores a precomputed mapping from producer expressions to consumer modules.

- `record_module_externals(module)` extracts every external value referenced by
  `module` (including nested operands) and records the dependency.
- `is_externally_used(expr, owning_module, *, exclude_push=True)` returns
  `True` when the expression is consumed by a different module. The optional
  `exclude_push` flag mirrors the legacy API and suppresses `FIFOPush`
  expressions when requested.

### `build_external_usage_index(modules: Iterable[ModuleBase])`

Convenience helper that instantiates an :class:`ExternalUsageIndex`, feeds it
all provided modules, and returns the populated registry.

### `expr_externally_used(expr: Expr, exclude_push: bool, index: ExternalUsageIndex | None = None)`

Backwards-compatible facade that either defers to the supplied registry or
falls back to scanning `expr.users`. All existing call sites continue to operate
without modification, while new code can supply a precomputed index for better
performance.

## Usage

The Verilog FIFO metadata pass now invokes `build_external_usage_index` ahead of
its IR walk and passes the registry into `FIFOAnalysisVisitor`. This ensures
cross-module exposure checks are consistent, avoids repeatedly scanning the full
module list, and removes the need for ad-hoc introspection of other modules'
`externals` dictionaries within the visitor itself.
