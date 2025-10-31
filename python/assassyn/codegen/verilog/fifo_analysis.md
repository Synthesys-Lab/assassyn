# FIFO Analysis Pre-pass

## Summary

The analysis module performs a read-only traversal of Assassyn IR before any Verilog code
is emitted. The helper `collect_fifo_metadata` drives a visitor that mirrors the dumper’s
predicate semantics and records every `FIFOPush`/`FIFOPop` interaction, FINISH intrinsic,
async call, and cross-module exposure into immutable metadata structures. The resulting
`module_metadata` map and `FIFORegistry` are passed to `CIRCTDumper` at construction time,
ensuring downstream phases observe a stable snapshot without manipulating mutable global
state during code emission.

## Exposed Interfaces

### `collect_fifo_metadata`

```python
def collect_fifo_metadata(
    sys: SysBuilder,
    modules: Sequence[Module] | None = None,
) -> tuple[dict[Module, ModuleMetadata], FIFORegistry]:
    """Traverse modules and build FIFO metadata."""
```

**Explanation**

This helper orchestrates the pre-pass that produces FIFO metadata for Verilog code
generation.

1. **Module Selection**: If `modules` is `None`, the helper walks every module and
   downstream module in the system. Otherwise it analyses only the supplied modules,
   allowing incremental workflows to refresh subsets and merge the new results into an
   existing cache.
2. **Visitor Execution**: Instantiates `FIFOAnalysisVisitor` with a fresh
   `FIFORegistry` and a mutable `dict[Module, ModuleMetadata]`. The visitor walks each
   module body, recording push/pop interactions, FINISH intrinsics, async calls, and any
   valued expression that must be exposed outside the module. Predicates are read directly
   from the base `Expr` snapshot (`expr.meta_cond`), so the stored metadata contains raw IR values.
3. **Metadata Construction**: For every visited module the helper creates a new
   `ModuleMetadata` whose `ModuleFIFOView` references the shared registry and whose
   `ModuleExposure` aggregates array, async-trigger, and general value exposures. Recorded
   `FIFOPush`/`FIFOPop` expressions are owned by the registry and re-used inside the module
   view so predicates and expression handles stay in sync for all consumers.
4. **Result Delivery**: Returns `(module_metadata, fifo_registry)` for the caller to feed
   into `CIRCTDumper`. The helper never mutates the caller’s existing metadata, making it
   safe to run in parallel with other analyses or to layer partial refreshes on top of
   cached data.

Consumers typically call `collect_fifo_metadata(sys)` before creating a `CIRCTDumper` for
full system emission. Tests or incremental tooling can analyse a subset of modules and
stitch the returned dictionaries into their own caches.

## Internal Helpers

### `FIFOAnalysisVisitor`

`FIFOAnalysisVisitor` subclasses the generic IR `Visitor` and overrides only `visit_expr`.
It receives two collaborators:

- A shared `FIFORegistry` that owns every recorded `FIFOPush`/`FIFOPop` expression.
- A mutable `dict[Module, ModuleMetadata]` populated on demand.

`visit_expr` handles four categories:

1. **FIFO interactions** – `FIFOPush` / `FIFOPop` nodes register their expressions in
   `FIFORegistry` and the per-module `ModuleFIFOView`, capturing predicates from
   the expression snapshot (`expr.meta_cond`). When a pop’s value escapes its defining module the visitor also
   records a value exposure so downstream stages can surface the produced data without
   revisiting the IR.
2. **FINISH intrinsics** – append the `Intrinsic.FINISH` expressions themselves to
   `ModuleMetadata.finish_sites` so downstream wiring can expose finish outputs without
   mutating state during emission.
3. **Async calls** – append `AsyncCall` expressions to `ModuleMetadata.calls` and record
   trigger exposure metadata (grouped per callee) with the associated predicate.
4. **Exposure candidates** – arrays and valued expressions used outside the module are
   captured in `ModuleExposure` structures so cleanup can emit wiring without revisiting
   the IR.

Traversal of module bodies is delegated to the base visitor, keeping the class compact and
ensuring new IR constructs automatically flow through analysis as long as they surface as
expressions.
