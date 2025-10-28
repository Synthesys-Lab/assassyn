# Array Ownership Model

## Summary

This document introduces the ownership metadata carried by every IR array. The
owner descriptor records which architectural unit is responsible for the array
contents and how downstream infrastructure should treat the storage. Replacing
the coarse `ArrayKind` enum with structured ownership objects lets the IR,
Verilog backend, simulator, and tooling query precise provenance information
without hard-coded buckets.

## Owner Taxonomy

Two concrete owner descriptors are currently defined. Both implement a common
interface that exposes a `category` string for quick filtering and helper
methods for downstream logic.

### `RegisterOwner`

- **Purpose**: Represents register arrays that belong to a pipeline module or
  global system context.
- **Fields**:
  - `module: ModuleBase | None` – The module that instantiated the array. This
    is `None` for arrays declared while no module context is active (for example
    system-level scratch space).
- **Behaviour**:
  - Verilog metadata collectors treat `RegisterOwner` arrays as regular
    multi-port register banks.
  - The simulator materialises these arrays as mutable `Array<T>` instances and
    pre-allocates ports based on module access patterns.

### `MemoryOwner`

- **Purpose**: Encodes arrays that are private to a `MemoryBase` subclass.
- **Fields**:
  - `memory: MemoryBase` – Reference to the owning memory instance. Downstream
    logic inspects the concrete subclass (SRAM or DRAM) to decide whether the
    storage is synthesised or abstracted.
  - `role: Literal["payload", "dout"]` – Distinguishes the backing payload from
    auxiliary buffers. `payload` arrays emulate the actual RAM contents.
    `dout` arrays (currently SRAM-only) act as the one-word read data latch.
- **Behaviour**:
  - Verilog passes skip payload arrays because dedicated SRAM/DRAM generators
    wire them, but they still expose `dout` buffers as standard registers.
  - The simulator materialises SRAM payloads so it can execute the simple black
    box, but it skips DRAM payloads and proxies them through the Ramulator2
    interface.

## Ownership Lifecycle

- **Assignment**: `RegArray` automatically assigns a `RegisterOwner` tied to the
  current module context. Memory modules call `RegArray(..., owner=MemoryOwner)`
  to override the default and associate payload buffers with the correct memory.
- **Mutation**: Ownership descriptors are immutable dataclasses. The only way to
  change ownership is through `Array.assign_owner`, which enforces type safety
  and is intended for tightly controlled refactors (for example, cloning an
  array into a new module).
- **Introspection**: `Array.owner` exposes the descriptor. Convenience checks
  such as `array.owner.category == "memory"` or
  `isinstance(array.owner, MemoryOwner)` allow downstream code to branch on
  semantics without relying on enum members.

## Downstream Integration

- **Verilog Backend**:
  - `ArrayMetadataRegistry` filters out `MemoryOwner(role="payload")` arrays so
    they do not receive generic multi-port wiring.
  - `generate_sram_control_signals` recognises `MemoryOwner` payloads when
    synthesising SRAM handshake logic and exposes `dout` arrays as regular
    registers.
  - System/top-level assembly checks the owner category when deciding which
    arrays require write-back wiring.
- **Simulator**:
  - Iterates over arrays and skips DRAM payload owners, delegating to the
    `MemoryInterface`. SRAM payloads remain materialised to back the simplified
    SRAM model.
  - Ownership metadata is preserved so debug tooling can report which module or
    memory produced each array.

## Extension Guidelines

- Prefer extending `MemoryOwner.role` for new memory-side buffers (for example,
  FIFOs) instead of introducing additional enums.
- If a future feature needs a different ownership semantic (e.g. external
  accelerators exporting arrays), add a dedicated dataclass to capture the
  required context rather than overloading existing roles.
- Whenever ownership transfer is required, implement it in a dedicated helper
  that records the transition. Avoid reassigning owners ad-hoc inside module
  code—this breaks the provenance guarantees the simulator and codegen rely on.
