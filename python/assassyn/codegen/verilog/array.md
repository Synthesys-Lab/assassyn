# Array Metadata Utilities

This module centralises collection and consumption helpers for Verilog array metadata.  It replaces the ad-hoc dictionaries that previously lived in `design.py` and `system.py`, making shared multi-port array handling easier to reason about and test.

## Summary

`array.py` provides an `ArrayMetadataRegistry` that owns every `ArrayMetadata` instance produced during system analysis.  The registry walks all modules once, records which modules read or write each IR array, assigns deterministic read/write port indices, and offers helper accessors used by code generation passes.  Downstream generators no longer need to coordinate mutable dictionaries; they query the registry instead.

## Exposed Interfaces

### `ArrayMetadataRegistry`

```python
class ArrayMetadataRegistry:
    """Collects and serves metadata for shared IR arrays."""
```

**collect(self, sys)**

- Invokes a full-system scan to populate the registry.
- Ignores arrays whose owner is a memory instance and `array.is_payload(owner)` returns `True`, because those are emitted as dedicated memory modules.
- Records writers via `Array.get_write_ports()` and assigns sequential write-port indices.
- Iterates each module body directly (thanks to the flattened IR described in [`DONE-remove-block`](../../../../dones/DONE-remove-block.md)) and records every `ArrayRead` / `ArrayWrite` expression, assigning read-port indices and user membership on first sighting.

**metadata_for(self, array) -> Optional[ArrayMetadata]**

- Returns the metadata object for the given IR array, or `None` when the array is ignored (e.g. memory payload identified by ownership metadata).

**write_port_index(self, array, module) -> Optional[int]**

- Retrieves the writer port index assigned to `module` for `array`.  Returns `None` when the module does not write the array.

**read_port_indices(self, array, module) -> List[int]**

- Returns the list of read-port indices used by `module`.  The list is empty when the module does not read the array.

**read_port_count(self, array) -> int**

- Shortcut returning the total number of read ports for `array`.

**write_port_count(self, array) -> int**

- Shortcut returning the total number of write ports for `array`.

**read_port_index_for_expr(self, expr) -> Optional[int]**

- Looks up the global read-port index allocated to a specific `ArrayRead` expression.  This is used by expression lowering and cleanup when referencing the generated wires.

**users_for(self, array) -> List[Module]**

- Returns the deterministic list of modules that either read or write the array.  This drives module port generation and top-level wiring.

**arrays(self) -> Iterable[Array]**

- Iterates over every array tracked by the registry in collection order.

## Internal Helpers

The registry keeps a reverse lookup from `ArrayRead` nodes to `(array, port_index)` tuples and ensures that each module is only registered once per array.  Duplication is avoided by checking whether a module or expression has already been seen before appending to the metadata structures.

## Project-specific Knowledge Required

- [`ArrayMetadata`](./metadata.md) – Structure describing per-array usage.
- [`generate_system`](./system.md) – Calls `collect()` during system analysis to prime the registry.
- [`cleanup.post_generation`](./cleanup.md) & [`module.generate_module_ports`](./module.md) – Consumers that rely on the registry for deterministic wiring and port declarations.
- [`DONE-remove-block`](../../../../dones/DONE-remove-block.md) – Documents the block removal that enables direct body iteration without the dumper helper.
