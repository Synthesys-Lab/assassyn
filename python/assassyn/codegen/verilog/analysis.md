# FIFO Analysis Pre-pass

## Summary

The analysis module performs a read-only traversal of Assassyn IR before any Verilog code
is emitted. The helper `collect_fifo_metadata` drives a visitor that mirrors the dumper’s
predicate semantics and records every array read/write, FIFO push/pop, FINISH intrinsic,
async call, and cross-module exposure into a unified `InteractionMatrix`. The resulting
`module_metadata` map and matrix are passed to `CIRCTDumper` at construction time,
ensuring downstream phases observe a stable snapshot without manipulating mutable global
state during code emission.

The implementation resides in `python/assassyn/codegen/verilog/analysis.py`.

`collect_external_metadata(sys)` complements the FIFO pass by aggregating
external module information into `ExternalRegistry`, recording external classes,
instance ownership, and cross-module reads before code generation begins.

`InteractionMatrix`, `ModuleMetadata`, and `InteractionKind` are imported from the
`python.assassyn.codegen.verilog.metadata` package (implemented across
`metadata.core`, `metadata.module`, and related submodules) but remain available via the
legacy `metadata` namespace for callers.

## Exposed Interfaces

### `collect_fifo_metadata`

```python
def collect_fifo_metadata(
    sys: SysBuilder,
    modules: Sequence[Module] | None = None,
) -> tuple[dict[Module, ModuleMetadata], InteractionMatrix]:
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
   `InteractionMatrix` and a mutable `dict[Module, ModuleMetadata]`. The visitor walks each
   module body, recording array and FIFO interactions, FINISH intrinsics, async calls, and any
   valued expression that must be exposed outside the module. Predicates are read directly
   from the base `Expr` snapshot (`expr.meta_cond`), so the stored metadata contains raw IR values.
3. **Metadata Construction**: For every visited module the helper asks the matrix for the
   module view and builds a `ModuleMetadata` carrying value exposures, async calls, FINISH sites,
   and the shared `ModuleInteractionView`. Recorded `FIFOPush`/`FIFOPop` and array expressions are
   owned by the matrix and referenced by both module and resource projections so predicates and handles
   stay in sync for all consumers.
4. **Result Delivery**: Returns `(module_metadata, interactions)` for the caller to feed
   into `CIRCTDumper`. Before returning, the helper calls `freeze()` on every
   `ModuleMetadata` and on the shared matrix (including its async ledger), converting the mutable accumulators
   into immutable tuples so downstream phases observe a stable snapshot. The helper never
   mutates the caller’s existing metadata, making it safe to run in parallel with other
   analyses or to layer partial refreshes on top of cached data.

Consumers typically call `collect_fifo_metadata(sys)` before creating a `CIRCTDumper` for
full system emission. Tests or incremental tooling can analyse a subset of modules and
stitch the returned dictionaries into their own caches.

## Internal Helpers

### `FIFOAnalysisVisitor`

`FIFOAnalysisVisitor` subclasses the generic IR `Visitor` and overrides only `visit_expr`.
It receives two collaborators:

- A shared `InteractionMatrix` that owns every recorded interaction and exposes the async ledger.
- A mutable `dict[Module, ModuleMetadata]` populated on demand.

`visit_expr` handles four categories:

1. **FIFO interactions** – `FIFOPush` / `FIFOPop` nodes register their expressions in
   the matrix, which simultaneously updates the module-facing buckets and the FIFO resource bucket while
   capturing predicates from the expression snapshot (`expr.meta_cond`). When a pop’s value escapes its defining module the visitor also
   records a value exposure so downstream stages can surface the produced data without revisiting the IR.
2. **FINISH intrinsics** – append the `Intrinsic.FINISH` expressions themselves to
   `ModuleMetadata.finish_sites` so downstream wiring can expose finish outputs without
   mutating state during emission.
3. **Async calls** – append `AsyncCall` expressions to `ModuleMetadata.calls` and record
   trigger exposure metadata in the matrix’s `async_ledger`, preserving per-callee groupings together with the associated predicate.
4. **Exposure candidates** – valued expressions used outside the module are captured directly on the module metadata so cleanup can emit wiring without revisiting the IR, while array interactions flow into the matrix buckets shared with array-aware emitters.

Traversal of module bodies is delegated to the base visitor, keeping the class compact and
ensuring new IR constructs automatically flow through analysis as long as they surface as
expressions.
