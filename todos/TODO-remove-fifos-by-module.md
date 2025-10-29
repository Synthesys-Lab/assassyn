# TODO: Remove `FIFORegistry._fifos_by_module` duplication

## Section 1: Goal

Eliminate the per-module FIFO bookkeeping map stored in `FIFORegistry._fifos_by_module` because `ModuleMetadata.fifo` (the `ModuleFIFOView`) already captures the module ↔ FIFO relationships. Streamlining this state should shrink the surface area for divergence, simplify registry cleanup, and make module-scoped queries the single source of truth.

## Section 2: Action Items

### Foundational Analysis

1. Re-read the FIFO metadata documentation and recent refactor notes (`python/assassyn/codegen/verilog/metadata.md`, `cleanup.md`, `design.md`, and `dones/DONE-verilog-fifo-metadata-refactor.md`) to refresh expected invariants for module-scoped and FIFO-scoped views.
2. Audit the Python sources for every caller that depends on `_fifos_by_module` via `FIFORegistry.channels_for_module` or `clear_for_module` (`python/assassyn/codegen/verilog/metadata.py`, `cleanup.py`, `design.py`, unit tests) and catalogue the exact semantics they require (iteration ordering, deduplication, clearing behaviour).
3. Confirm that `ModuleFIFOView` consistently records the FIFO ports per module by inspecting where `ModuleMetadata.record_fifo_interaction` is invoked (_expr handlers) and whether any code bypasses the view (e.g., direct registry access during async lowering). Carefully document that `ModuleMetadata.fifo` maintains the set of FIFO ports along with the pushes/pops issued by the *owning* module, while `FIFORegistry` tracks every interaction (push or pop) keyed by the FIFO port itself, aggregating contributions from all modules.

### Documentation First

4. Update `python/assassyn/codegen/verilog/metadata.md` to document `ModuleFIFOView` as the authoritative per-module index and note the removal of `_fifos_by_module`, including guidance on how to iterate module FIFOs going forward.
5. Adjust `python/assassyn/codegen/verilog/cleanup.md` (and `design.md`/`top.md` if they mention `channels_for_module`) so the prose describes retrieving FIFO interactions through `ModuleMetadata.fifo` instead of the registry map.
6. Stage these documentation changes before touching code, keeping them as an isolated logical chunk for future commits.

### Tests Before Implementation

7. Extend `python/unit-tests/codegen/test_fifo_metadata.py`:
   - Assert that `ModuleMetadata.fifo.ports` and any new helpers yield the same port set previously provided by `channels_for_module`.
   - Add coverage to ensure re-visiting a module clears stale registry entries without relying on `_fifos_by_module`.
8. If additional behaviours depended on `channels_for_module` (e.g., cleanup wiring), add or update targeted tests (potentially a new fixture under `python/unit-tests/codegen/`) to exercise the new iteration path and fail with the current implementation.
9. Run the focused tests to capture their failure mode prior to code changes (`source setup.sh && pytest python/unit-tests/codegen/test_fifo_metadata.py [...]`) and record observations for reference during implementation.

### Implementation

10. Enhance `ModuleFIFOView` (`python/assassyn/codegen/verilog/metadata.py`) with utility methods that surface module-associated FIFO metadata directly (e.g., `iter_channels()` yielding `(Port, FIFOMetadata)` by consulting the registry on demand) and ensure `ports` remains deduplicated.
11. Remove `_fifos_by_module` from `FIFORegistry`:
    - Drop its initialization and all mutations in `record_push`/`record_pop`.
    - Rewrite `channels_for_module` to delegate to a supplied `ModuleFIFOView` (or deprecate/replace it with `ModuleFIFOView.iter_channels()`), updating type hints and docstrings accordingly.
12. Refactor `FIFORegistry.clear_for_module` so it operates using the module’s recorded FIFO ports rather than the removed map:
    - Before clearing, have `CIRCTDumper.visit_module` capture the existing `ModuleMetadata` (if any) to obtain `metadata.fifo.ports`.
    - Provide the collected port set to `clear_for_module`, ensuring stale interactions are removed and empty `FIFOMetadata` objects are pruned.
13. Update all call sites:
    - `cleanup.py` should switch to iterating `metadata.fifo.iter_channels()` (or equivalent) and drop manual filtering by module.
    - Any other consumer of `channels_for_module` must be updated to rely on the module view, sharing helper utilities where appropriate to avoid ad-hoc rewrites.
14. Sweep the codebase for leftover references to `_fifos_by_module` or the legacy API, ensuring naming/comments align with the new flow.

### Validation & Wrap-up

15. Re-run the enriched FIFO-focused pytest targets to confirm the new approach satisfies expectations (`source setup.sh && pytest python/unit-tests/codegen/test_fifo_metadata.py [...]`).
16. Execute the broader regression suite (`source setup.sh && make test-all`) to guard against unintended knock-on effects in other codegen paths.
17. Capture the outcome in a new summary entry under `dones/` (e.g., `dones/DONE-remove-fifos-by-module.md`), documenting the benefits, edge cases handled, and follow-up ideas (such as caching strategies if iteration becomes hot).
18. Stage and commit changes following the project’s commit message standard, sourcing the environment before running `git commit` so pre-commit hooks execute.

## Section 3: Open Questions / Coordination Points

1. Do any external consumers (outside Verilog codegen) rely on `FIFORegistry.channels_for_module`? If so, coordinate updates or provide adapter utilities before removal.
2. Will module re-visitation occur without an existing `ModuleMetadata` entry (e.g., single-module generation)? Ensure the revised cleanup path gracefully handles missing historical views.
3. Does dropping `_fifos_by_module` introduce performance regressions for large systems? Measure iteration costs and consider caching within `ModuleFIFOView` if necessary.

## Section 4: Risks & Mitigations

1. **Incomplete Cleanup**: Failing to supply the correct FIFO port set to `clear_for_module` could leave stale interactions; mitigate with regression tests that revisit modules multiple times and assert interaction counts.
2. **API Breakage**: Consumers expecting `channels_for_module` may break; mitigate by introducing transitional helpers (or deprecation warnings) and updating documentation/tests in lock-step.
3. **Ordering Differences**: Switching iteration sources might reorder emitted Verilog; mitigate by preserving deterministic ordering (e.g., sort ports or rely on insertion order) and verifying via existing golden outputs/tests.

## Section 5: Summary Checklist

- [x] Documentation updated to reflect the single-source module FIFO view (Items 4–6)
- [x] Tests augmented to guard cleanup/iteration semantics without `_fifos_by_module` (Items 7–9)
- [x] Implementation removes `_fifos_by_module`, updates registry helpers, and rewires call sites (Items 10–14)
- [x] Regression and full test suites executed successfully (Items 15–16)
- [x] Work documented in `dones/` and staged for commit per guidelines (Items 17–18)
