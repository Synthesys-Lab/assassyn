# Array Ownership Model

## Summary

Every IR array exposes an `owner` attribute that points to the runtime object
responsible for the storage. Rather than wrapping the information in dedicated
descriptor classes, the `owner` is simply a reference to the defining module,
memory instance, or `None` when no explicit context exists. Downstream passes
inspect the referenced object (most notably the concrete `MemoryBase`
subclasses) to determine how the storage should be handled during code
generation and simulation. Consumers use `Array.is_payload(...)` to encapsulate
the payload detection logic that previously relied on ad hoc identity checks.

## Ownership Semantics

### Register Arrays

- Arrays instantiated at the system scope carry `owner = None`.
- Arrays created inside a module store a reference to that `ModuleBase`
  instance. The Verilog backend and simulator use the module reference to derive
  port ownership and naming.

### Memory-backed Arrays

- Memory constructors (`SRAM`, `DRAM`, or future subclasses) assign `owner = self`
  to their internal arrays.
- Payload buffers are identified via `array.is_payload(memory)` (accepting either
  the memory class or instance). The helper normalises the argument into a
  single validation path before checking ownership, so unsupported inputs raise
  the same `TypeError` message no matter which form was used. Only payload
  buffers are skipped by generic register plumbing.
- Auxiliary registers such as the SRAM `dout` latch also reference the memory
  instance through `owner`, but they are treated like ordinary registers because
  they are not the payload array.

## Ownership Lifecycle

- **Assignment**: `RegArray` records the current module (if any) automatically.
  Memory modules call `RegArray(..., owner=self)` to override the default once
  their base class initialisation has completed.
- **Mutation**: `Array.assign_owner` remains available for controlled updates and
  enforces that the owner is either a `ModuleBase`, a `MemoryBase`, or `None`.
- **Introspection**: Use `array.owner` directly. To differentiate payloads from
  other arrays, call `array.is_payload(memory_cls_or_instance)` instead of
  repeating manual identity checks.

## Downstream Integration

- **Verilog Backend**
  - `ArrayMetadataRegistry` filters out payload buffers via `array.is_payload(memory)`,
    keeping the identity logic centralised.
  - SRAM-specific passes detect payload access by delegating to the helper,
    allowing the handshake logic to route to dedicated memory signals.
- **Simulator**
  - The simulator enumerates arrays and skips DRAM payload buffers by checking
    `array.is_payload(DRAM)`; SRAM payloads remain materialised for the behavioural model.
  - Register-owned arrays (module references or `None`) continue to be allocated
    as mutable simulator arrays.

## Extension Guidelines

- Future memories that introduce additional internal arrays should expose named
  attributes similar to `_payload` and consult those identities when branching.
- If a new component needs to own arrays without being a `ModuleBase` or
  `MemoryBase`, extend `assign_owner` to accept that type and document the
  identity-based checks downstream users should apply.
- Avoid adding fragile string tags; rely on concrete Python objects so that
  ownership changes can be reasoned about using object identity.
