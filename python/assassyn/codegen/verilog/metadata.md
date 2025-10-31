# Verilog Code Generation Metadata

This module provides metadata structures for tracking information collected for Verilog code generation that need to be referenced in later compilation phases.  FIFO metadata is now populated by a dedicated analysis pre-pass that runs before any code is emitted, ensuring every downstream consumer observes a stable snapshot of push/pop activity.

## Summary

The metadata module defines dataclasses that hold information about modules discovered during a dedicated FIFO analysis pass that precedes code emission. The pass is orchestrated by [`collect_fifo_metadata`](./analysis.md), which uses `FIFOAnalysisVisitor` to walk the requested modules, reading each interaction’s predicate carry (`expr.meta_cond`) so the recorded predicates mirror those consumed by the dumper.  Later phases consume the frozen metadata, eliminating the need for runtime bookkeeping and avoiding mismatches caused by incremental mutation.  A dedicated `FIFORegistry` keeps a FIFO-keyed view in lockstep with the per-module metadata so every consumer can choose the lookup that best fits its wiring task without recomputing groupings.

## FIFO Analysis Pre-pass

`FIFOAnalysisVisitor` performs a read-only walk of module bodies before `CIRCTDumper.visit_module` runs.  The visitor reuses the dumper’s rvalue formatting logic (via a lightweight analysis shim) to evaluate predicates and, by storing the original `Value` objects captured in the predicate snapshot, guarantees the exact same predicate normalisation as the code generator.  The visitor is intentionally small—only `visit_expr` is overridden—so traversal mirrors the runtime dumper and remains easy to audit.  Every invocation of `collect_fifo_metadata` returns fresh data structures; callers that want to refresh a subset of modules can request just those modules and merge the results without mutating previously produced registries.

Key responsibilities:

1. Instantiate `ModuleMetadata` for every visited module and attach a `ModuleFIFOView` pointing at the shared registry.
2. Record every `FIFOPush`/`FIFOPop` interaction by storing the raw expressions in both the shared registry and the per-module view, preserving the original predicate `Value` captured in `expr.meta_cond`.

## Exposed Interfaces

### `ArrayMetadata`

```python
@dataclass
class ArrayMetadata:
    """Metadata describing how an IR array is used across the system."""
```

**Explanation**

`ArrayMetadata` captures every detail required to synthesise a shared multi-port memory wrapper for an IR `Array`.  The structure is produced by [`array.py`](./array.md) while the system is analysed, and then consumed by `design.py`, `module.py`, `cleanup.py`, and `top.py` when they need to emit per-port wires, assignments, or PyCDE modules.

**Fields:**

- `array: Array` – Reference to the IR array this metadata describes.
- `write_ports: Dict[Module, int]` – Deterministic mapping from writer modules to their assigned write-port indices.  These indices are stable across the run so that every consumer can agree on signal names such as `_w_port0`.
- `read_ports_by_module: Dict[Module, List[int]]` – Per-module list of read-port indices used when a module exposes address wires for the shared reader.
- `read_order: List[Tuple[Module, ArrayRead]]` – Ordered catalogue of every `ArrayRead` encountered.  The index in this list is the global read-port number.
- `read_expr_port: Dict[ArrayRead, int]` – Reverse lookup from a specific read expression to its global port index; used by expression code generation and cleanup to reference the right `*_rdata_portN` signal.
- `users: List[Module]` – Unique list of modules that touch the array (read or write).  This drives both module port generation and top-level wiring.

**When Metadata is Populated:**

`ArrayMetadata` instances are emitted by [`ArrayMetadataRegistry.collect`](./array.md) during `generate_system`.  The registry records writers via `Array.get_write_ports()`, then iterates each module's `body` list directly (see [`DONE-remove-block`](../../../../dones/DONE-remove-block.md)) to find `ArrayRead` / `ArrayWrite` expressions, assigning read-port indices in first-seen order while skipping arrays whose owner is a memory instance and for which `array.is_payload(owner)` is `True`; those are emitted separately.

**How Metadata is Consumed:**

- [design.py](./design.md) – Builds PyCDE array wrapper classes with the correct number of read/write ports.
- [module.py](./module.md) – Declares per-module ports for array reads/writes by querying the registry.
- [cleanup.py](./cleanup.md) – Routes module-level signals into shared array writers using the recorded port indices.
- [top.py](./top.md) – Emits global wire declarations and instance connections for every shared array.

The registry exposes helper methods (`write_port_index`, `read_port_indices`, `read_port_index_for_expr`, `users_for`) that all consumers use instead of recomputing the data.  This eliminates the ad-hoc dictionaries previously scattered across `design.py` and `system.py`.

### `ModuleMetadata`

```python
@dataclass
class ModuleMetadata:
    """Metadata collected during module code generation."""
```

**Explanation**

This dataclass tracks module-level facts discovered during the analysis pre-pass and
exposes them to later phases.  Its FIFO information is a *view* layered on top of the
global registry populated during the pre-pass.  The module view is the authoritative
record of which FIFO ports a module touches; the registry only keeps the complementary
FIFO-keyed aggregation.  Starting with the metadata consolidation, every other
code-generation concern that previously mutated the dumper during emission (FINISH flags,
async-call tracking, array/value exposure bookkeeping) is now captured here as immutable
snapshots.

**Fields**

- `module: Module` – Owning module used to filter registry lookups.
- `finish_sites: Tuple[Intrinsic, ...]` – Ordered list of FINISH intrinsics recorded during
  analysis; each entry preserves the intrinsic expression so cleanup/top wiring can query
  `expr.meta_cond` and gate `self.finish` without mutating dumper state.
- `calls: List[AsyncCall]` – Populated during analysis when async calls are encountered
  in the module body.  Emission simply reads the preserved list.
- `fifo: ModuleFIFOView` – Module-scoped view that references registry-owned FIFO
  interactions.  It keeps per-module lists of those shared objects without duplicating
  their contents.
- `exposures: ModuleExposure` – Aggregated array/async/value exposure information
  collected by the pre-pass.  Cleanup and module port generation consume these immutable
  structures instead of relying on `CIRCTDumper.expose()`.

**Convenience Properties**

- `pushes` – Returns `FIFOPush` expressions for this module by projecting
  `fifo.pushes`.
- `pops` – Returns `FIFOPop` expressions for this module by projecting `fifo.pops`.

**When Metadata is Populated**

1. `FIFOAnalysisVisitor` ensures a `ModuleMetadata` instance exists for the module,
   clearing any stale FIFO interactions and wiring the metadata to the shared registry.
2. The visitor reads each interaction’s predicate carry (`expr.meta_cond`) so the stored
   expressions retain the original `Value` guard captured at analysis time.
3. Each fifo push/pop encountered during the pre-pass stores the raw expression in the
   registry and registers it with the module’s `ModuleFIFOView`.
4. When valued expressions require exposure (array writes/reads, async triggers, general
   outputs) or FINISH/async-call nodes are encountered, the analysis visitor records them
   directly in `ModuleMetadata`, trusting the expressions’ `meta_cond` metadata to capture
   predicate carries alongside the expression handles.  FINISH intrinsics are stored as raw
   intrinsic expressions instead of toggling a boolean.
5. During subsequent code generation the same `ModuleMetadata` object remains read-only;
   emission simply queries `ModuleMetadata` for FIFO interactions, FINISH flags, async
   calls, and exposure data without mutating state.

**How Metadata is Consumed**

- **Top-level harness generation** ([top.py](/python/assassyn/codegen/verilog/top.md)):
  Reads `metadata.fifo.interactions_by_kind[FIFOPush]` (mirrored by the
  `metadata.fifo.pushes` shortcut) and `metadata.finish_sites` to compute FIFO wiring and
  finish exposure.
- **Module port generation** ([module.py](/python/assassyn/codegen/verilog/module.md)):
  Uses `metadata.fifo` for handshake ports and `metadata.exposures.values` to declare
  `expose_*` / `valid_*` outputs.
- **Cleanup wiring** ([cleanup.py](/python/assassyn/codegen/verilog/cleanup.md)):
  Iterates `metadata.exposures.arrays`, `metadata.exposures.values`, and
  `metadata.exposures.async_triggers` to emit final signal assignments without walking
  expressions or mutating dumper state.
- **Finish collection**: Iterates `finish_sites` recorded during the pre-pass, formatting
  predicates at emit time instead of rebuilding conditionals on the dumper.
- **Performance benefit**: Maintains O(1) lookups with predicate context intact while
  eliminating the runtime `_exposes` dictionary.

**Future Extensions**

The `ModuleMetadata` structure can still be extended with additional flags such as
`has_wait_until` or `array_usage`.  Exposure metadata already centralises the wiring
surface, and the new `finish_sites` snapshots keep FINISH handling consistent without
runtime mutation, making future additions (e.g., external memory handshakes) straightforward.

### `ModuleExposure`

```python
@dataclass
class ModuleExposure:
    arrays: Dict[Array, ArrayExposure]
    values: List[Expr]
    async_triggers: Dict[Module, List[AsyncCall]]
```

This container replaces the dumper’s `_exposes` dictionary.  Entries are populated during
analysis and stay immutable afterwards.  Each collection stores raw IR handles, relying on
the expressions’ `meta_cond` metadata to recover predicate carries when formatting the
final wiring.

While the visitor runs these containers remain mutable so it can append entries without
recreating the owning structures.  Property access therefore returns defensive tuples
generated from the backing lists.  When `ModuleMetadata.freeze()` executes, the lists are
converted in place to tuples, the containers refuse further mutation, and subsequent reads
return the same tuple object.  The pattern both prevents accidental post-pass mutations and
avoids per-access allocations once the snapshot is considered authoritative.

### `ArrayExposure`

```python
@dataclass
class ArrayExposure:
    array: Array
    writes_by_module: Dict[Module, Tuple[ArrayWrite, ...]]
    reads: Tuple[ArrayRead, ...]
```

`ArrayExposure` groups all array interactions performed by the owning module.  Writes are
bucketed by source module so cleanup can derive per-port enable/data muxes while reads are
listed in first-seen order for index wiring.  Downstream passes format each write’s
predicate using `write.meta_cond`, mirroring the guard captured when the expression was
materialised without duplicating the condition object.

### Value Exposures and Async Triggers

- **Values** – `ModuleExposure.values` stores bare `Expr` instances whose results must be
  exposed because another module depends on them (`expr_externally_used(expr, True)`).  The
  predicate for each expression is obtained from `expr.meta_cond` during cleanup.
- **Async triggers** – `ModuleExposure.async_triggers` maps callee modules to lists of
  `AsyncCall` expressions that contribute to trigger counters.  Cleanup sums each call’s
  predicate via `call.meta_cond` when driving `<callee>_trigger` wires.

### Finish Sites

`ModuleMetadata.finish_sites` is recorded as a tuple of `Intrinsic` expressions whose opcode
is `Intrinsic.FINISH`.  Each intrinsic already carries its predicate metadata, so downstream
consumers simply read `expr.meta_cond` without touching auxiliary wrapper objects.

**Project-specific Knowledge Required:**

- Understanding of [CIRCTDumper state management](/python/assassyn/codegen/verilog/design.md)
- Knowledge of [intrinsic code generation](/python/assassyn/codegen/verilog/_expr/intrinsics.md)
- Reference to [top-level harness generation](/python/assassyn/codegen/verilog/top.md)
- Understanding of [visitor pattern](/python/assassyn/ir/visitor.md)

### `ModuleFIFOView`

```python
class ModuleFIFOView:
    """Module-scoped view over registry-owned FIFO interactions."""
```

`ModuleFIFOView` keeps track of every FIFO port a module touched and provides filtered
access to the registry-owned expressions.  It is the authoritative source for per-module
FIFO sets:

- `ports` – Iterable of FIFO ports the module interacted with (preserving insertion order).
- `interactions_by_kind` – Mapping from the interaction class (`FIFOPush` / `FIFOPop`) to
  the tuples of expressions recorded for the module.  `pushes` / `pops` are thin
  convenience projections that return the corresponding bucket from this mapping.
- `interactions_for(port)` – Returns the expressions recorded for `port` that originate
  from the owning module, letting consumers wire ready/valid signals without re-filtering
  the registry.
- `iter_channels()` – Iterates `(Port, FIFOMetadata, Sequence[Expr])` triples, exposing the
  registry-owned channel metadata alongside the module’s filtered expressions without
  relying on registry-maintained module maps.  Direction is recovered via
  `isinstance(expr, FIFOPush)` / `FIFOPop`, predicates via `expr.meta_cond`, and producer
  modules via the module metadata that owns the view (or `expr.parent` as a fallback).

`ModuleMetadata.freeze()` snaps the module-scoped FIFO lists to tuples in lockstep with the
shared `FIFORegistry` so emitters can cache lookups without repeatedly allocating
defensive copies.  Any attempt to register a new interaction after freezing raises a
`RuntimeError`, surfacing accidental late mutations immediately.

### `FIFOMetadata`

```python
class FIFOMetadata:
    """Per-FIFO channel metadata owned by the registry."""
```

Each FIFO port is associated with a `FIFOMetadata` instance that stores ordered tuples of
raw expressions grouped by interaction kind:

- `interactions_by_kind` – Dictionary keyed by the interaction class (`FIFOPush` or
  `FIFOPop`) whose values are the tuples of expressions captured during analysis.
- `pushes` / `pops` – Convenience projections that surface the push/pop tuples without
  forcing callers to index into the mapping.
- `record_interaction()` – Adds the expression to the appropriate bucket, preserving
  encounter order so downstream muxing logic remains deterministic.

### `FIFORegistry`

```python
class FIFORegistry:
    """Global FIFO metadata index keyed by FIFO ports."""
```

The registry is the single owner of FIFO interaction data:

- `record_interaction()` – Normalises both push and pop events by appending the raw
  expression to the port’s `FIFOMetadata`, returning the same expression for module-level
  bookkeeping.
- `record_push()` / `record_pop()` – Thin compatibility wrappers that delegate to
  `record_interaction()` while retaining the original call sites in analysis code.
- `metadata_for(port)` – Fetch (or lazily create) the `FIFOMetadata` container for `port`.

Calling `FIFORegistry.freeze()` walks every `FIFOMetadata` instance, converts internal
lists to tuples, and flips their mutation guards.  This mirrors the per-module freeze so
any post-pass attempt to write into the registry is rejected and repeated property access
returns the same immutable tuple.

Because both the module-level view and downstream consumers reference the same expression
objects, predicates and handles stay perfectly in sync without two indices drifting apart.
