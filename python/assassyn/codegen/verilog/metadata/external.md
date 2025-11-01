# External Metadata Helpers

`external.py` owns the registry that captures metadata for external SystemVerilog
modules. The registry collects external classes, instance ownership, and
cross-module reads so downstream emitters can wire producer and consumer modules
without re-walking the IR.

## Exposed Interfaces

### `ExternalRead`

Immutable record containing:

- `expr` – the `PureIntrinsic(EXTERNAL_OUTPUT_READ)` node.
- `producer` / `consumer` – modules producing and consuming the external value.
- `instance` – the `ExternalIntrinsic` emitting the output.
- `port_name` – the output port identifier.
- `index_operand` – optional index used when reading register-style outputs.

### `ExternalRegistry`

Mutable accumulator that offers `record_*` helpers during analysis and exposes
read-only views after `freeze()`:

- `record_instance(instance, owner)` – register an external instance and its owning module.
- `record_cross_module_read(...)` – capture a consumer reading from another
  module’s external output.
- `classes` – tuple of discovered external classes, preserving analysis order.
- `instance_owners` – mapping from `ExternalIntrinsic` to owner `Module`.
- `cross_module_reads` – tuple of all recorded reads.
- `reads_for_consumer(module)` / `reads_for_instance(instance)` /
  `reads_for_producer(module)` – convenient projections for emitters.
- `owner_for(instance)` – lookup helper used during code generation.

Calling `freeze()` converts all internal storage to tuples and `MappingProxyType`
instances, preventing accidental mutation during emission.
