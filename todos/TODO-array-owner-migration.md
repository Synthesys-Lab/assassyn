# TODO: Replace `ArrayKind` with the Array Owner Design

## Section 1: Goal

Retire the newly introduced `ArrayKind` enum and migrate the entire stack to the “array owner” design where each IR array carries structured ownership metadata. The new model must let IR builders, memory modules, simulator, and Verilog backend derive behaviour (payload handling, port exposure, materialisation) from owner descriptors rather than hard-coded enum buckets, while preserving existing functionality and tests.

## Section 2: Action Items

### Analysis & Preparation

1. **Inventory current `ArrayKind` usage**
   - Trace the enum across `python/assassyn/ir/array.py`, `python/assassyn/ir/memory/{base,sram,dram}.py`, `python/assassyn/codegen/verilog/*`, and `python/assassyn/codegen/simulator/simulator.py`.
   - Collect all documentation (`python/assassyn/ir/*.md`, `docs/design/internal/{pipeline,simulator}.md`, Verilog docs) and tests (`python/unit-tests/test_array_kind.py`, related CI tests) that reference `ArrayKind`.

2. **Clarify the owner design requirements**
   - Locate or draft the owner design spec (expected in `docs/design/internal/`) describing owner categories (e.g., `RegisterOwner`, `MemoryOwner` with payload/dout roles) and invariants.
   - Enumerate behaviour expectations: which owners should surface in Verilog metadata, how simulator materialises arrays, and any constraints on owner reassignment.

3. **Assess compatibility impacts**
   - Identify external consumers (tooling scripts, metadata dump readers, simulator runtime) that expect `ArrayKind`; decide whether to provide temporary shims or coordinated updates.
   - Cross-check open TODOs (e.g., `todos/TODO-doc-fix-memory.md`) to avoid conflicting edits.

### Documentation (draft before code)

4. **Author owner design documentation**
   - Add `docs/design/internal/array-ownership.md` (or update if it exists) detailing the motivation, owner taxonomy, lifecycle, and extension guidance.
   - Update `python/assassyn/ir/array.md` and memory docs (`python/assassyn/ir/memory/{base,sram,dram}.md`) to introduce the owner concept and explain how each builder assigns owners.
   - Refresh `docs/design/internal/{pipeline,simulator}.md` and Verilog docs (`python/assassyn/codegen/verilog/*.md`) to replace `ArrayKind` discussion with owner-driven behaviour.

5. **Document migration guidance**
   - Add upgrade notes (e.g., `docs/design/internal/changelog.md` or similar) outlining how downstream users should transition from `Array.kind` checks to owner queries.

### Testing (write before implementation)

6. **Create owner-based regression tests**
   - Replace `python/unit-tests/test_array_kind.py` with `test_array_owner.py` covering:
     - Default owner for plain `RegArray` (likely `RegisterOwner` tied to the defining module).
     - SRAM/DRAM builders tagging payload and `dout` arrays with appropriate owner descriptors.
     - Metadata queries reflecting owner information.
   - Ensure tests initially fail while the enum remains in place.

7. **Guard invariants with tests**
   - Add tests asserting owner descriptors are immutable or change only via sanctioned APIs.
   - Include simulator/codegen-focused tests verifying owner-driven inclusion/exclusion (e.g., only register-owned arrays appear in generic metadata).

### Implementation

8. **Introduce owner descriptor primitives**
   - Implement owner data structures in `python/assassyn/ir/array.py` (e.g., `ArrayOwner` base with subclasses or enums paired with context objects).
   - Extend `RegArray` and related helpers to assign default owners; provide utilities for memory builders to create payload/dout owners while capturing owning module/memory references.

9. **Remove `ArrayKind` from the IR**
   - Delete the enum, `kind` property, and related validation from `array.py`.
   - Update memory modules to use owner helpers instead of setting `kind`.

10. **Propagate owner logic through codegen and simulator**
    - Refactor Verilog codegen (`array.py`, `_expr/array.py`, `cleanup.py`, `system.py`, `top.py`, `design.py`) to branch on owner descriptors for metadata filtering, port creation, and cleanup.
    - Update `python/assassyn/codegen/simulator/simulator.py` and any helper modules to rely on owners when deciding which arrays to materialise or skip, preserving existing behaviour (e.g., keep SRAM payload storage, skip DRAM payloads).
    - Adjust metadata exporters to embed owner information and, if necessary, provide backward-compatible fields during transition.

11. **Update public exports and remove legacy paths**
    - Drop `ArrayKind` from `python/assassyn/ir/__init__.py` exports and re-export new owner primitives.
    - Remove deprecated collections like `sram_payload_arrays` if the owner design makes them redundant.

### Validation & Wrap-up

12. **Run focused test suites**
    - `source setup.sh && python -m pytest python/unit-tests/test_array_owner.py`
    - Execute updated backend/simulator regression tests that assert owner behaviour.

13. **Execute full regressions**
    - `source setup.sh && python -m pytest python/ci-tests/test_sram.py python/ci-tests/test_dram.py`
    - `source setup.sh && make test-all`

14. **Finalize documentation and summaries**
    - Re-read all modified docs to ensure terminology consistency and cross-links.
    - Record outcomes, insights, and follow-up ideas in `dones/DONE-array-owner.md`.

15. **Commit workflow**
    - Stage changes incrementally (docs → tests → code), capturing meaningful intermediate commits when appropriate.
    - Follow the commit message guideline for the final change (e.g., `[IR][Docs] Replace ArrayKind with owner-based provenance`).

## Section 3: Open Questions

1. **Owner schema specifics** – Do owners include both the creating module and a role string, or a richer structure (e.g., memory type + direction)? Confirm before implementation.
2. **External consumer expectations** – Which tooling relies on `Array.kind`, and what compatibility guarantees must we provide?
3. **Lifecycle constraints** – Are there scenarios where ownership must transfer (e.g., array cloning), and how should the API support or forbid that?

## Section 4: Potential Follow-ups

1. **Static validation** – Add lint passes ensuring owner descriptors are assigned for every array and match allowed producer/consumer relationships.
2. **Builder ergonomics** – Provide helper factories (e.g., `make_sram_payload_array`) to reduce manual owner assignment in frontends.
3. **Telemetry** – Add optional debug logs or metadata dumps showing owner information for easier debugging.

## Section 5: Completion Checklist

- [x] Owner design documented and cross-referenced.
- [x] `ArrayKind` removed from IR and codegen paths.
- [x] Simulator and Verilog backends rely solely on owner metadata.
- [x] Owner-focused tests pass alongside full regression suites.
- [x] Summary captured in `dones/DONE-array-owner.md`.

## Section 6: Summary

- Documented the ownership model in `docs/design/internal/array-ownership.md` and refreshed all affected IR/codegen docs to reference owner descriptors.
- Replaced the `ArrayKind` enum with `RegisterOwner`/`MemoryOwner` dataclasses, updated IR helpers, memory builders, and exports accordingly.
- Updated Verilog backend and simulator code paths to branch on ownership metadata instead of enum checks while preserving SRAM/DRAM-specific behaviour.
- Replaced `test_array_kind.py` with `test_array_owner.py` to cover default owners, memory payload tagging, metadata filtering, and immutability APIs.
- Ran `python -m pytest python/unit-tests/test_array_owner.py`, `python -m pytest python/ci-tests/test_sram.py python/ci-tests/test_dram.py`, and `make test-all` after sourcing `setup.sh`; all suites pass.
