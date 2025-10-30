# FIFO Analysis Pre-pass

## Summary

The FIFO analysis module performs a read-only traversal of Assassyn IR before any Verilog
code is emitted. The helper `collect_fifo_metadata` drives a lightweight
`FIFOAnalysisVisitor` that mirrors the dumper’s predicate semantics and records every
`FIFOPush`/`FIFOPop` interaction into immutable metadata structures. The resulting
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
   `FIFORegistry`, a mutable `dict[Module, ModuleMetadata]`, and a predicate formatter
   that shares the dumper’s naming rules. The visitor visits each module body and records
   push/pop interactions while maintaining predicate context with `PredicateStack`.
3. **Metadata Construction**: For every visited module the helper creates a new
   `ModuleMetadata` whose `ModuleFIFOView` references the shared registry. Recorded
   `FIFOInteraction` instances are owned by the registry and re-used inside the module
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
It receives three collaborators:

- A shared `FIFORegistry` that owns every `FIFOInteraction`.
- A mutable `dict[Module, ModuleMetadata]` populated on demand.
- A predicate formatter that exposes `dump_rval(expr, with_namespace, module_name)` so
  `PUSH_CONDITION` intrinsics and their predicates match the strings produced during code
  emission.

`visit_expr` handles three cases:

1. `Intrinsic.PUSH_CONDITION` – formats the guard expression with the shared predicate
   formatter and pushes it onto `PredicateStack`.
2. `Intrinsic.POP_CONDITION` – pops the most recent predicate frame.
3. `FIFOPush` / `FIFOPop` – records interactions in `FIFORegistry` and in the
   per-module `ModuleFIFOView`, using the current predicate string supplied by the stack.

Traversal of module bodies is delegated to the base `Visitor`, keeping the class small and
ensuring new IR constructs automatically flow through the pass as long as they surface as
expressions.
