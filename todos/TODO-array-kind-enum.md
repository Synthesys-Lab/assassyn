# TODO: Array Kind Enumeration for Memory-Aware Codegen

## Section 1: Goal

Introduce an explicit `ArrayKind` enum on `python/assassyn/ir/array.py:Array` so the IR, documentation, and Verilog backend can distinguish register arrays from SRAM/DRAM payloads (and similar special cases) without ad-hoc checks. Default every array to `REG`, tag memory payloads during instantiation (`python/assassyn/ir/memory/sram.py`, `python/assassyn/ir/memory/dram.py`), and update the Verilog codegen to rely on the enum instead of bespoke collections when applying memory-specific rules.

## Section 2: Action Items

### Analysis & Preparation

1. **Catalogue existing array usage patterns**
   - Walk through `python/assassyn/ir/array.py`, `python/assassyn/ir/memory/base.py`, `python/assassyn/ir/memory/{sram,dram}.py`, and `python/assassyn/codegen/verilog/` modules (`array.py`, `cleanup.py`, `system.py`, `_expr/array.py`, `top.py`).
   - Confirm how `_payload`, `dout`, and general `RegArray` instances flow into codegen today, and identify any other modules that create special-purpose arrays (e.g., search for `RegArray(` across the repo to ensure no additional kinds are needed).

2. **Cross-check design and documentation expectations**
   - Revisit `python/assassyn/ir/array.md`, `python/assassyn/ir/memory/{base,sram,dram}.md`, and the relevant design docs (`docs/design/internal/pipeline.md`, `docs/design/arch/arch.md`, `docs/design/internal/simulator.md`) to note current descriptions of register arrays and memory payloads.
   - Review `python/assassyn/codegen/verilog/*.md` (especially `array.md`, `cleanup.md`, `system.md`, `top.md`) to understand how SRAM payload handling is currently documented.

### Documentation Updates (must precede implementation)

3. **Document the new enum on the IR side**
   - Extend `python/assassyn/ir/array.md` with a subsection describing `ArrayKind`, the default `REG` behavior, and how `Array.kind` is consumed downstream.
   - Update examples or API snippets to mention the optional `kind` parameter (if added) and how memory modules override it.

4. **Refresh memory module docs**
   - Amend `python/assassyn/ir/memory/base.md`, `python/assassyn/ir/memory/sram.md`, and `python/assassyn/ir/memory/dram.md` to state explicitly which `ArrayKind` values their payload buffers receive and why.

5. **Align Verilog backend documentation**
   - Update `python/assassyn/codegen/verilog/{array.md,cleanup.md,system.md,top.md}` (and any other impacted guides) so they reference `Array.kind` instead of bespoke SRAM payload sets when explaining control-signal generation and metadata filtering.
   - If the design docs (`docs/design/internal/pipeline.md`, `docs/design/internal/simulator.md`) discuss SRAM payload discovery, add a note that the enum now drives that logic.

### Testing (write before feature implementation)

6. **Add unit coverage for the enum**
   - Create `python/unit-tests/test_array_kind.py` (or extend an existing suite) to assert:
     - A plain `RegArray` defaults to `ArrayKind.REG`.
     - `SRAM` construction tags `_payload` (and `dout`, if applicable) with the expected kind.
     - `DRAM` construction tags `_payload` appropriately.
   - The new test should fail before the feature work and be run via `source setup.sh && python -m pytest python/unit-tests/test_array_kind.py`.

7. **Add backend regression coverage**
   - Write or extend a focused test (e.g., under `python/unit-tests/` or `python/ci-tests/`) that drives the Verilog dumper over a minimal design containing both a register array and an SRAM/DRAM payload.
   - Assert that only `REG` arrays appear in `ArrayMetadataRegistry` output, while memory payload arrays are excluded based on `Array.kind`, and that special-case wiring (e.g., `mem_dataout`) still occurs.
   - Execute this test prior to implementation to confirm it fails with the current behavior.

### Implementation

8. **Define the enum and plumb it through array construction**
   - Introduce an `Enum` (likely `enum.Enum`) named `ArrayKind` inside `python/assassyn/ir/array.py`, with at least `REG`, `SRAM_PAYLOAD`, `SRAM_DOUT`, and `DRAM_PAYLOAD` entries (adjust names if discovery warrants).
   - Update the `Array` constructor to accept and store a `kind` attribute (defaulting to `REG`), expose it as a property, and ensure `__repr__`/`as_operand` remain stable.
   - Decide whether `RegArray` should accept an optional `kind` keyword or whether memory modules should set `array.kind` post-instantiation; implement accordingly and reflect that choice in docs/tests.

9. **Tag memory-specific arrays**
   - In `python/assassyn/ir/memory/base.py`, `python/assassyn/ir/memory/sram.py`, and `python/assassyn/ir/memory/dram.py`, assign the appropriate `ArrayKind` values to `_payload` and any auxiliary buffers (e.g., `dout`).
   - Audit other subsystems (e.g., simulator or IP modules) for special arrays that might deserve distinct kinds, adjusting the enum if new categories are required.

10. **Export the enum for consumers**
   - Update `python/assassyn/ir/__init__.py` (and its `.md` counterpart if present) to re-export `ArrayKind`, ensuring external code can query kinds without deep imports.

### Codegen Integration

11. **Refactor payload detection in the dumper**
   - Replace usages of `dumper.sram_payload_arrays` and `_is_payload_owner` guards with checks against `array.kind` across:
     - `python/assassyn/codegen/verilog/array.py` (`ArrayMetadataRegistry.collect`, helper methods)
     - `python/assassyn/codegen/verilog/cleanup.py`
     - `python/assassyn/codegen/verilog/system.py`
     - `python/assassyn/codegen/verilog/_expr/array.py`
     - Any other helpers that special-case SRAM arrays (e.g., `top.py`, `design.py`, `utils.py`).
   - Keep backwards-compatible behavior (e.g., lazily populate `sram_payload_arrays` from `kind` if other subsystems still consult it) or remove the redundant state if it is no longer needed.

12. **Guard Verilog port generation with the enum**
   - Ensure only arrays tagged `REG` (or other non-memory kinds deemed appropriate) are exposed as generic array ports, while memory payload kinds trigger the dedicated SRAM/DRAM wiring paths.
   - Verify that auxiliary buffers like `SRAM`'s `dout` continue to receive the correct signals after the refactor.

13. **Update metadata structures if needed**
   - If `ArrayMetadataRegistry` or related classes serialize kind information, extend their dataclasses/structures and serialization paths accordingly so downstream tools can inspect the new field when needed.

### Validation & Wrap-up

14. **Run targeted tests**
   - `source setup.sh && python -m pytest python/unit-tests/test_array_kind.py`
   - Run the new/updated backend regression test(s).

15. **Run broader regressions**
   - Execute the existing SRAM/DRAM CI tests (`source setup.sh && python -m pytest python/ci-tests/test_sram.py`, and any DRAM-specific suites) to confirm no regressions.
   - Finish with `source setup.sh && make test-all` prior to finalizing.

16. **Finalize documentation and commit**
   - Re-read all touched docs for consistency, ensure enum names match code, and add cross-links where helpful.
   - Stage changes and commit with a descriptive message (e.g., `IR: add ArrayKind and teach Verilog codegen about memory payloads`) following the project's commit message guideline.

## Section 3: Unclear Aspects Requiring Human Input

1. **Granularity of kinds** – Do we need separate enum values for SRAM payload buffers versus their read-data registers, or is a single `MEMORY_PAYLOAD` sufficient? Confirm preferred taxonomy before locking the API.
2. **External consumers** – Check whether any external tooling (e.g., simulator backends, Rust bindings under `tools/`) assumes arrays are uniformly registers and would require simultaneous updates.
3. **Attribute interactions** – Clarify whether existing `Array.attr` entries (e.g., `Array.FULLY_PARTITIONED` in examples) should interplay with the new `kind`, or remain orthogonal.

## Section 4: Potential Follow-ups / Enhancements

1. **Expose kind in simulator artifacts** – Consider propagating `Array.kind` into the simulator runtime or metadata dumps so downstream tooling can leverage the distinction.
2. **Kind-aware factory helpers** – Evaluate adding convenience constructors (e.g., `MemoryArray`) if more array kinds emerge, to avoid manual tagging in multiple modules.
3. **Static analysis hooks** – Add linting or builder-time checks that prevent mixing incompatible kinds (e.g., writing to an SRAM payload outside its owning module), now that the enum is available.

---

## Section 5: Completion Checklist

- [x] Documented `ArrayKind` across IR, memory modules, and Verilog backend guides.
- [x] Added `python/unit-tests/test_array_kind.py` covering IR defaults, memory tagging, and metadata filtering.
- [x] Implemented `ArrayKind` enum, plumbed it through `RegArray`, memory constructors, and re-exported it.
- [x] Updated Verilog codegen and simulator to gate payload handling via `Array.kind`, eliminating `sram_payload_arrays`.
- [x] Ran `python -m pytest python/unit-tests/test_array_kind.py`, `python -m pytest python/ci-tests/test_sram.py python/ci-tests/test_dram.py`, and `make test-all`.
