# TODO: Introduce FIFO-indexed metadata mirror for Verilog codegen

## Section 1: Goal

Deliver a redundant metadata view keyed by FIFO ports (reusing `FIFOMetadata` entries) alongside the existing module-indexed metadata so downstream Verilog codegen stages can look up push/pop context either by module or by FIFO without re-deriving groupings, enabling a simpler cleanup pass.

## Section 2: Action Items

### Foundational Analysis

1. Re-read the FIFO-related documentation to refresh constraints and invariants before proposing code changes: `python/assassyn/codegen/verilog/metadata.md`, `python/assassyn/codegen/verilog/cleanup.md`, `python/assassyn/codegen/verilog/design.md`, `python/assassyn/codegen/verilog/top.md`, and `dones/DONE-remove-fifo-expose.md`. Confirm whether any other design docs (e.g. `docs/design/internal/module.md`) rely on the current module-only metadata description.
2. Audit the current Python sources to map where FIFO metadata is consumed: `python/assassyn/codegen/verilog/_expr/array.py`, `cleanup.py`, `module.py`, `top.py`, and any helper utilities under `python/assassyn/codegen/verilog`. Note the exact expectations each consumer has (e.g., grouping pushes by `expr.fifo`, relying on `ModuleMetadata.fifo.pushes` being a flat list).
3. Inspect the existing unit tests (`python/unit-tests/codegen/test_fifo_metadata.py`, `test_fifo_cleanup_metadata.py`, `test_fifo_pop_metadata.py`) to catalogue assertions that must continue to pass and identify gaps where the FIFO-keyed mirror should be validated.

### Documentation First

4. Update `python/assassyn/codegen/verilog/metadata.md` to describe the new FIFO-keyed registry, its lifecycle within `CIRCTDumper`, and how it relates to `ModuleMetadata`.
5. Revise `python/assassyn/codegen/verilog/cleanup.md` (and, if needed, `design.md`/`top.md`) so the prose documents the simplified cleanup flow: metadata lookup happens via both module and FIFO keys, eliminating ad-hoc grouping.
6. If the high-level design docs under `docs/design/internal` reference FIFO metadata, refresh those passages to mention the symmetry and clarify which phases consume each view.
7. Stage and (later) commit the documentation tweaks before any functional changes to comply with the development guideline ordering.

### Tests Before Features

8. Extend `python/unit-tests/codegen/test_fifo_metadata.py` to assert the new FIFO-keyed registry exists, returns the same `FIFOMetadata` instance for a FIFO as recorded under the module, and does not accumulate stale entries after regenerating a module.
9. Enhance `python/unit-tests/codegen/test_fifo_cleanup_metadata.py` (or add a focused new test if necessary) to exercise cleanup via the FIFO-indexed mirror: check that the dumper uses the FIFO-level metadata without rebuilding `defaultdict` groupings, and verify push/pop predicates and muxing remain intact.
10. Add regression coverage for cross-module scenarios if absent (e.g., a producer module pushing into another module’s FIFO) to ensure the FIFO-keyed structure gracefully hosts entries from multiple modules while the module-indexed view still filters correctly.
11. Run the updated tests once written to confirm they fail against the current implementation, documenting the expected failure messages.

### Implementation

12. In `python/assassyn/codegen/verilog/metadata.py`, introduce a helper registry (e.g., `FIFORegistry`) that maintains a `Dict[FIFO, FIFOMetadata]`, and adjust `FIFOMetadata.record_push/pop` to return the created metadata entries so both indices can share the same objects rather than duplicating state.
13. Update `ModuleMetadata` so it can reference FIFO-scoped metadata objects produced by the registry while preserving the existing `.fifo` API (e.g., keep a list façade that aggregates per-module pushes/pops by storing handles returned from the registry).
14. Modify `CIRCTDumper.__init__` and module lifecycle methods in `python/assassyn/codegen/verilog/design.py` to instantiate and maintain the FIFO-indexed registry, including bookkeeping (per-module FIFO sets) that allows safe re-visitation of modules without leaving stale entries in the global map.
15. Adjust the FIFO expression handlers in `python/assassyn/codegen/verilog/_expr/array.py` to register pushes/pops into both the module metadata (for backwards compatibility) and the new FIFO registry, ensuring the same `FIFOMetadata` object is shared where appropriate.
16. Refactor `python/assassyn/codegen/verilog/cleanup.py` to rely on the FIFO-indexed map instead of rebuilding per-port groupings:
    - Retrieve FIFO metadata from the registry, filter entries belonging to the current module, and emit push/pop wiring using the pre-grouped data.
    - Drop the local `defaultdict` construction while keeping predicates, mux logic, and naming identical.
17. Review other consumers (`module.py`, `top.py`, docs) to decide whether they should adopt the FIFO-keyed lookup or continue using the module view; update them if the new registry allows simplifying duplicated grouping logic.
18. Ensure the registry is accessible where needed (e.g., expose it on `CIRCTDumper`) and add type hints/comments for clarity.

### Validation & Wrap-up

19. Re-run the FIFO-focused unit tests first: `source setup.sh && pytest python/unit-tests/codegen/test_fifo_metadata.py python/unit-tests/codegen/test_fifo_cleanup_metadata.py python/unit-tests/codegen/test_fifo_pop_metadata.py`, confirming the new expectations now pass.
20. Execute the broader regression suite `source setup.sh && make test-all` to guard against collateral regressions.
21. Summarize the work in a new `dones/DONE-fifo-metadata-mirror.md` (or similarly named) entry detailing achieved symmetry, design decisions, and follow-up ideas, then stage and commit it.
22. Stage and commit code/documentation changes in logical increments with guideline-compliant messages, invoking `pre-commit` hooks via `git commit` after sourcing the environment.

## Section 3: Open Questions / Coordination Points

1. Should the FIFO-indexed registry expose immutable snapshots for downstream phases, or is live mutation acceptable? Decide based on how `cleanup.py` and `top.py` consume metadata.
2. How should FIFO metadata be cleared when re-visiting modules individually (outside full-system generation) to avoid duplicate entries? Evaluate whether module-level teardown hooks are required.
3. Are there external consumers (beyond Verilog codegen) expecting only the module-indexed view? If so, catalogue them to ensure the new mirror does not diverge behaviour or require adapter APIs.

## Section 4: Risks & Mitigations

1. **Data Divergence**: Maintaining two indices increases the risk of them drifting; mitigate by sharing `FIFOPushMetadata`/`FIFOPopMetadata` instances and adding assertions/tests that both views stay in sync.
2. **Lifecycle Bugs**: Forgetting to purge FIFO entries when modules are regenerated could leak stale predicates; design explicit cleanup hooks in the dumper and cover them with tests.
3. **Downstream Breakage**: Refactoring `cleanup.py` to rely on the new registry might inadvertently change emitted code ordering; guard with golden/regression tests that inspect generated Verilog snippets.
4. **Documentation Debt**: Multiple docs reference the old module-only view; ensure all relevant guides are updated early so they remain authoritative during implementation.

## Section 5: Summary Checklist

- [x] Reviewed design docs and existing FIFO metadata consumers (Items 1-3)
- [x] Updated Verilog metadata/design/cleanup/top documentation for the FIFO-indexed registry (Items 4-7)
- [x] Extended FIFO unit tests, including cross-module coverage and pre-change failure verification (Items 8-11)
- [x] Implemented `FIFORegistry`, module metadata wiring, dumper lifecycle hooks, and cleanup refactor (Items 12-18)
- [x] Ran focused FIFO pytest targets and full `make test-all` regression (Items 19-20)
- [x] Prepared to document work in `dones/` and proceed to staging/commit (Items 21-22)
