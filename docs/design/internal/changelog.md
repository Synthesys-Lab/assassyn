# Internal Changelog

This document captures developer-facing migrations that affect internal users of
the Assassyn codebase. Each entry summarises the change, lists the impacted
interfaces, and outlines required follow-up for downstream consumers.

## Array Ownership Simplification

- **Summary**: `Array.kind`/`ArrayKind` remain deprecated, but ownership is now
  expressed directly through `Array.owner = ModuleBase | MemoryBase | None`
  instead of custom descriptor dataclasses.
- **Effective**: 2025-11-01 (array owner simplification refactor)
- **Affected Components**:
  - Code paths that previously imported `RegisterOwner`/`MemoryOwner`.
  - Backend utilities filtering arrays through `owner.role`.
  - Simulator helpers that skipped payload buffers.
- **Migration Guidance**:
  1. Stop importing `RegisterOwner` or `MemoryOwner`; they no longer exist.
  2. Use direct identity checks for general ownership and the `Array.is_payload` helper to encapsulate payload detection:
     - `owner is None` for top-level arrays.
     - `owner is module_instance` for module-scoped arrays.
     - `isinstance(owner, MemoryBase)` to detect memory-managed arrays.
     - Call `array.is_payload(owner)` (or `array.is_payload(SRAM)` / `DRAM`) to skip payload buffers while leaving auxiliary registers (e.g., SRAM `dout`) intact.
  3. When creating memory-managed arrays, pass `owner=self` from the memory
     constructor.
  4. Use `array.assign_owner(new_owner)` when ownership must change; the method
     validates the supplied object.
- **Deprecations**: `RegisterOwner`, `MemoryOwner`, and `ArrayOwner` aliases are
  removed. Code relying on `.role` must switch to identity checks as described
  above.
- **Testing Notes**: `python/unit-tests/test_array_owner.py` has been updated to
  cover the simplified semantics. Downstream tests that asserted descriptor
  types should adopt the new identity-based expectations.
