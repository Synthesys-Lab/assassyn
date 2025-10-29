# Verilog Code Generation Metadata

This module provides metadata structures for tracking information collected during Verilog code generation that needs to be referenced in later compilation phases.

## Summary

The metadata module defines dataclasses that hold information about modules discovered during the code generation pass. This metadata is populated incrementally as expressions are processed and later consumed during top-level harness generation, eliminating the need for redundant analysis passes.  A dedicated `FIFORegistry` keeps a FIFO-keyed view in lockstep with the per-module metadata so every consumer can choose the lookup that best fits its wiring task without recomputing groupings.

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

This dataclass tracks module-level facts discovered during code generation and exposes
them to later phases.  Its FIFO information is now a *view* layered on top of the global
registry rather than a duplicated structure.

**Fields**

- `module: Module` – Owning module used to filter registry lookups.
- `has_finish: bool = False` – Toggled by `codegen_intrinsic` when a FINISH intrinsic is
  encountered so the top-level harness knows which modules expose finish signals.
- `calls: List[AsyncCall]` – Populated by `codegen_async_call` when async calls are emitted.
- `fifo: ModuleFIFOView` – Module-scoped view that references registry-owned FIFO
  interactions.  It keeps per-module lists of those shared objects without duplicating
  their contents.

**Convenience Properties**

- `pushes` – Returns `FIFOPush` expressions for this module by projecting
  `fifo.pushes`.
- `pops` – Returns `FIFOPop` expressions for this module by projecting `fifo.pops`.

**When Metadata is Populated**

1. `CIRCTDumper.visit_module` initialises a `ModuleMetadata` instance for the current
   module, wiring it to the shared `FIFORegistry`.
2. `codegen_intrinsic` and `codegen_async_call` update `has_finish` and `calls`
   respectively.
3. [`_expr/array.py`](./_expr/array.md) records FIFO operations: the registry creates the
   shared `FIFOInteraction`, and the module view stores a reference to it together with the
   touched FIFO port.

**How Metadata is Consumed**

- **Top-level harness generation** ([top.py](/python/assassyn/codegen/verilog/top.md)):
  Reads `metadata.fifo.pushes` to compute FIFO depths and wiring.
- **Module port generation** ([module.py](/python/assassyn/codegen/verilog/module.md)):
  Uses `metadata.fifo` to determine which handshake ports are required.
- **Cleanup wiring** ([cleanup.py](/python/assassyn/codegen/verilog/cleanup.md)):
  Iterates `metadata.fifo.ports` and `interactions_for(port)` to emit valid/ready logic
  without re-scanning expressions.
- **Finish collection**: Uses `has_finish` to decide which modules surface finish outputs.
- **Performance benefit**: Maintains O(1) lookups with predicate context intact, while
  avoiding duplicated FIFO metadata.

**Future Extensions**

The `ModuleMetadata` structure can still be extended with additional flags such as
`has_wait_until` or `array_usage`.  The refactor only removes duplicated FIFO storage; the
extensibility story remains unchanged.

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
access to the shared interactions:

- `ports` – Iterable of FIFO ports the module interacted with.
- `pushes` / `pops` – Lists of `FIFOInteraction` objects produced by the module (references
  to the registry-owned entries).
- `interactions_for(port)` – Returns the interactions for `port` that originate from the
  owning module, letting consumers wire ready/valid signals without re-filtering the
  registry.

### `FIFOInteraction`

```python
@dataclass
class FIFOInteraction:
    module: Module
    expr: Union[FIFOPush, FIFOPop]
    predicate: str
```

This unified record replaces the redundant `FIFOPushMetadata` / `FIFOPopMetadata`
wrappers.  The interaction type is inferred from `expr`, and the predicate preserves the
conditional context observed during code generation.

### `FIFOMetadata`

```python
class FIFOMetadata:
    """Per-FIFO channel metadata owned by the registry."""
```

Each FIFO port is associated with a `FIFOMetadata` instance that stores ordered lists of
`FIFOInteraction` objects:

- `pushes` – Interactions whose expression is a `FIFOPush`.
- `pops` – Interactions whose expression is a `FIFOPop`.
- `record_interaction()` – Adds a new interaction to the appropriate list.
- `remove_module(module)` – Purges all interactions produced by `module`, keeping the
  registry consistent after regenerating a module.

### `FIFORegistry`

```python
class FIFORegistry:
    """Global FIFO metadata index keyed by FIFO ports."""
```

The registry is the single owner of FIFO interaction data:

- `record_push()` / `record_pop()` – Create a `FIFOInteraction`, append it to the port’s
  `FIFOMetadata`, and remember that the emitting module touched the port.
- `metadata_for(port)` – Fetch (or lazily create) the `FIFOMetadata` container for `port`.
- `channels_for_module(module)` – Iterate over `(Port, FIFOMetadata)` pairs for every FIFO
  the module interacted with.  [`cleanup.py`](./cleanup.md) uses this to wire valid/ready
  logic without consulting module-level mirrors.
- `clear_for_module(module)` – Remove all interactions emitted by `module` across every
  port and drop empty channel records.

Because both the module-level view and downstream consumers reference the same
`FIFOInteraction` objects, predicates and expression handles stay perfectly in sync without
two indices drifting apart.
