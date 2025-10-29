# TODO Simplify Verilog FIFO Metadata

## Context
- `python/assassyn/codegen/verilog/metadata.py` currently defines both `ModuleMetadata` and `FIFORegistry` with overlapping responsibilities and redundant FIFO entry wrappers (`FIFOPushMetadata` / `FIFOPopMetadata`).
- The module document (`python/assassyn/codegen/verilog/metadata.md`) still mirrors this structure, and downstream consumers/tests (`cleanup.py`, `module.py`, `python/unit-tests/codegen/test_fifo_metadata.py`, etc.) rely on the duplication.
- Requested improvements:
  1. Remove the redundant `FIFO{Push,Pop}Metadata` wrappers.
  2. Separate concerns between `ModuleMetadata` and `FIFORegistry` so their responsibilities are orthogonal instead of mirrored.
  3. Stop using string-based type annotations where the concrete types are already imported.

## Detailed Plan
1. **Deepen Understanding**
   - Re-read `python/assassyn/codegen/verilog/metadata.md`, `cleanup.md`, `module.md`, and `design.md` to map every documented consumer of FIFO metadata.
   - Grep call sites of `FIFOPushMetadata`, `FIFOPopMetadata`, and `fifo_by_port` to see how each module/test expects to interact with the metadata.
   - Capture the invariants that must keep holding (e.g., predicates preserved, registry clearing semantics, backwards compatibility requirements highlighted in `python/unit-tests/codegen/test_fifo_metadata.py`).

2. **Draft the Refined Design**
   - Sketch a canonical FIFO interaction record (single dataclass, e.g. `FIFOInteraction`) that stores the module, operation kind (push/pop), predicate, and expression.
   - Decide how `FIFORegistry` becomes the sole owner of these interaction objects keyed by `Port`, exposing filtered views (`pushes`, `pops`) while staying mutation-safe.
   - Define what `ModuleMetadata` should retain: likely lightweight references such as a set of FIFO ports used plus per-kind views that proxy back to the registry, eliminating duplicated state.
   - Plan how registry clearing (`clear_for_module`) will remove/share interactions without the mirrored lists and ensure module re-visits stay idempotent.
   - Outline the approach for proper typing without string annotations, e.g. importing the real IR types under `TYPE_CHECKING` plus `from __future__ import annotations` so runtime imports stay cheap.

3. **Update Documentation First**
   - Rewrite the FIFO sections in `python/assassyn/codegen/verilog/metadata.md` to describe the new interaction structure, the division of responsibility between `ModuleMetadata` and `FIFORegistry`, and the helper APIs each exposes.
   - Adjust any other affected docs (`module.md`, `cleanup.md`, potentially `top.md`) so their narratives match the refined API before touching code.
   - Note any follow-up documentation TODOs uncovered while drafting (e.g., if other modules implicitly rely on ordering guarantees).

4. **Implement Metadata Refactor**
   - Introduce the unified FIFO interaction dataclass and remove `FIFOPushMetadata` / `FIFOPopMetadata`.
   - Refactor `FIFORegistry` to own and expose these interactions per FIFO port, keeping mutation helpers that return stable references for consumers.
   - Rework `ModuleMetadata` to reference the registry-owned data (e.g., maintain per-module port sets or cached filtered lists) so it no longer stores duplicate `FIFOMetadata` aggregates.
   - Replace string annotations with direct type references (leveraging `from __future__ import annotations` and the existing `TYPE_CHECKING` imports).
   - Update any helper methods (e.g., `pushes`, `pops`, `register_fifo_push/pop`) to reflect the new data flow or remove them if obsolete.

5. **Propagate API Changes**
   - In `python/assassyn/codegen/verilog/cleanup.py`, replace the current inline `if` filtering with a dedicated helper that iterates the registry’s FIFO-indexed metadata so every FIFO is processed uniformly.
   - Adjust all call sites in `cleanup.py`, `module.py`, `design.py`, `_expr/array.py`, and any other modules uncovered in Step 1 to use the refined API.
   - Update `CIRCTDumper` (or other owners) to work with the orthogonalized `ModuleMetadata`/`FIFORegistry` split and ensure metadata clearing logic remains correct.
   - Revise `python/unit-tests/codegen/test_fifo_metadata.py` (and add new assertions if needed) to validate the new data model, including cross-module sharing and registry clearing.

6. **Testing**
   - Run targeted unit tests for codegen metadata: `source setup.sh && pytest python/unit-tests/codegen/test_fifo_metadata.py` to iterate quickly.
   - Once passing, execute the broader regression suite `source setup.sh && make test-all` to verify no regressions elsewhere.

7. **Documentation & Knowledge Capture**
   - Double-check that updated docs compile clean narratives with the new API (including any diagrams or tables if needed).
   - Summarize the work in `dones/DONE-verilog-fifo-metadata-refactor.md`, highlighting the architectural shift, benefits, and suggested follow-ups (e.g., opportunities for further registry validation or exposing more helper utilities).

8. **Commit Strategy**
   - Stage documentation updates and, if practical, commit them separately once the plan’s direction is validated.
   - After implementing code/test changes and confirming lint/tests pass, stage everything and commit with a meaningful message per the project’s git standard, ensuring the pre-commit hook runs (requires `source setup.sh` beforehand).

---

## Summary

0. **Goal**: Simplified FIFO metadata so `ModuleMetadata` exposes a module-scoped view over the registry while the registry remains the single owner of per-port interactions.

1. **Action Items**
   - [x] Re-read design docs and inspect current FIFO metadata consumers.
   - [x] Drafted the refined data model and documented the new responsibilities.
   - [x] Updated metadata/cleanup/module/top documentation prior to code changes.
   - [x] Implemented the refactor in `metadata.py` and adjusted all call sites (cleanup, module generation, expression lowering, dumper init).
   - [x] Updated unit tests to exercise the registry-backed view and verified cleanup iterates `channels_for_module`.
   - [x] Ran targeted FIFO metadata tests followed by `make test-all`.

2. **Changes**
   - Introduced `FIFOInteraction`, rewrote `ModuleMetadata` to use `ModuleFIFOView`, and made `FIFORegistry` the sole owner of channel metadata (`python/assassyn/codegen/verilog/metadata.py`).
   - Reworked FIFO wiring to consume registry lookups directly and removed `fifo_by_port` (`python/assassyn/codegen/verilog/cleanup.py`, `_expr/array.py`, `design.py`).
   - Updated docs (`metadata.md`, `cleanup.md`, `module.md`, `top.md`, `design.md`) to describe the orthogonal split and new helper APIs.
   - Expanded FIFO metadata tests to assert the registry/module views stay aligned (`python/unit-tests/codegen/test_fifo_metadata.py`).

3. **Decisions & Insights**
   - Chose a unified `FIFOInteraction` with an `is_push` flag to avoid runtime type checks while keeping expression references intact.
   - Exposed `FIFORegistry.channels_for_module` so cleanup can operate purely on FIFO-indexed metadata and avoid ad-hoc filtering.
   - Kept module-level push/pop lists as references to registry-owned objects to preserve ordering guarantees without duplicating state.
