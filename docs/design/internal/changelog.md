# Internal Changelog

This document captures developer-facing migrations that affect internal users of
the Assassyn codebase. Each entry summarises the change, lists the impacted
interfaces, and outlines required follow-up for downstream consumers.

## Array Ownership Migration

- **Summary**: `Array.kind` and the `ArrayKind` enum have been replaced by
  structured ownership descriptors exposed via `Array.owner`.
- **Effective**: 2025-10-28 (array owner migration TODO completion)
- **Affected Components**:
  - IR builders and transformation passes that previously branched on
    `ArrayKind`.
  - Verilog backend helpers that filtered arrays using `Array.kind`.
  - Simulator utilities that skipped DRAM payload arrays.
  - Any external tooling importing `assassyn.ir.ArrayKind`.
- **Migration Guidance**:
  1. Replace imports of `ArrayKind` with `RegisterOwner`, `MemoryOwner`, or the
     generic `ArrayOwner` protocol.
  2. Instead of comparing `array.kind`, inspect `array.owner`:
     - Use `isinstance(array.owner, MemoryOwner)` to detect memory-managed
       arrays.
     - Check `array.owner.role == "payload"` to exclude memory payload buffers.
     - Optional: read `array.owner.memory` to differentiate SRAM vs DRAM
       behaviour.
  3. When creating arrays that should belong to a memory, pass an explicit
     `owner=MemoryOwner(...)` override to `RegArray`.
  4. Avoid mutating internal fields; call `array.assign_owner()` if ownership
     must change.
- **Deprecations**: `ArrayKind` is removed entirely; importing it now fails.
  Code that still references `array.kind` will raise `AttributeError`.
- **Testing Notes**: New regression tests under
  `python/unit-tests/test_array_owner.py` cover the ownership model. Downstream
  projects should add equivalent coverage if they extend array ownership.
