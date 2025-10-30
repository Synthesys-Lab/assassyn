# TODO: Slim down FIFO analysis entrypoint redundancy

## Section 1: Goal

Replace the current FIFO pre-pass scaffolding (`CIRCTDumper.run_fifo_analysis`, `_ensure_fifo_metadata`, incremental bookkeeping) with a lean analysis routine that traverses the IR exactly once, returns two immutable metadata dictionaries (per-module and per-FIFO), and hands those into `CIRCTDumper` during construction. The new flow should keep predicate fidelity and incremental regeneration guarantees without forcing the dumper to manage refresh logic at visit time.

## Section 2: Action Items

### Foundational Analysis

1. Re-read the relevant docs to ground the refactor: `python/assassyn/codegen/verilog/metadata.md`, `design.md`, `system.md`, `predicate.py/md`, `docs/design/internal/pipeline.md`, and the historical summary in `dones/DONE-fifo-prepass-metadata.md`. Capture all invariants consumers expect from FIFO metadata (ordering, predicate strings, module scoping, incremental updates).
2. Audit the current implementation path: `FIFOAnalysisVisitor.analyze`, `CIRCTDumper.run_fifo_analysis`, `_ensure_fifo_metadata`, `visit_module`, and the registry helpers in `metadata.py`. Identify which responsibilities truly belong to the visitor versus the dumper, and catalog any side effects (`dumper.current_module`, predicate stack resets, registry replacement) that must be eliminated or migrated.
3. Trace how tests exercise these APIs: walk through `python/unit-tests/codegen/test_fifo_metadata.py`, `test_fifo_cleanup_metadata.py`, and any fixtures calling `run_fifo_analysis`. Note the expectations around explicit invocation, selective module analysis, and metadata reuse between codegen passes so the new API can provide equivalent hooks.

### Proposed Architecture

4. Design a standalone helper (e.g., `collect_fifo_metadata(sys: SysBuilder, modules: Sequence[Module] | None = None)`) that:
   - Instantiates a `FIFOAnalysisVisitor` with only the data it needs (predicate stack, registry references).
   - Walks the requested modules exactly once, returning `(module_metadata: dict[Module, ModuleMetadata], fifo_registry: FIFORegistry)` without mutating a `CIRCTDumper`.
   - Supports selective module traversal by clearing/rebuilding metadata entries just for the supplied modules while reusing previously computed results when possible.
5. Decide how `FIFOAnalysisVisitor` should expose results now that it no longer reaches into the dumper: plan the minimal constructor surface (likely the registry, module metadata dict, and a `dump_rval` callback). Document any helper closures or lightweight context objects required to compute predicate strings without the full dumper. Aim to mirror the pre-pass’ original simplicity by letting the visitor override only `visit_expr`, delegating traversal to the base class just like the former `_expr/array.py` handlers did before the HEAD refactor.
6. Plan the new `CIRCTDumper` lifecycle:
   - Accept precomputed `module_metadata` and `fifo_registry` through its constructor (with explicit typing).
    - Remove `_fifo_analysis`, `_fifo_analyzed_modules`, `run_fifo_analysis`, and `_ensure_fifo_metadata`; replace with simple assertions that injected metadata already covers every visited module.
   - Ensure downstream helpers (`cleanup_post_generation`, `generate_top_harness`, etc.) still reference the same registry objects they previously accessed on `self`.
7. Outline the top-level wiring changes:
   - Update `generate_design` / `generate_system` to call `collect_fifo_metadata` before instantiating the dumper and pass the returned dicts into `CIRCTDumper`.
   - Adjust any command-line or scripting entry points that currently rely on `run_fifo_analysis` or the implicit on-demand behaviour.

### Documentation First

8. Revise `python/assassyn/codegen/verilog/metadata.md`, `design.md`, `system.md`, and `docs/design/internal/pipeline.md` to describe the new analysis-first lifecycle, emphasising that metadata is computed prior to dumper creation and remains immutable during code emission.
9. Add or refresh documentation for `fifo_analysis.py` (create `fifo_analysis.md` if missing) to explain the visitor’s simplified role, its inputs/outputs, and how selective module analysis is handled without mutating the dumper.
10. Note any API removals (`run_fifo_analysis`, `_ensure_fifo_metadata`) in developer-facing docs or changelogs so users know to migrate to `collect_fifo_metadata`.

### Tests Before Implementation

11. Draft/adjust unit tests so they cover the new flow:
    - Update existing tests to use `collect_fifo_metadata` and verify the returned dicts contain the expected entries without needing a dumper instance.
    - Add regression tests for partial module re-analysis: run `collect_fifo_metadata` on the whole system, mutate/clear one module, re-run on just that module, and confirm both module and FIFO dicts reflect the refresh while other entries remain untouched.
    - Add a guard test ensuring `CIRCTDumper` raises a clear error if asked to visit a module missing from the injected metadata.
12. Execute the updated tests to confirm they currently fail (since the new helper does not yet exist) and document observed failures as a baseline.

### Implementation

13. Implement `collect_fifo_metadata` and reshape `FIFOAnalysisVisitor` to detach it from the dumper, keeping it to a single `visit_expr` override that records FIFO interactions using the shared predicate stack helper.
14. Rework `CIRCTDumper`:
    - Add constructor parameters for the metadata dicts and store them immutably.
    - Delete `_ensure_fifo_metadata`, `_fifo_analysis`, `_fifo_analyzed_modules`, and `run_fifo_analysis`.
    - Replace `visit_module`’s metadata acquisition with simple lookups that assume pre-seeded data, failing fast with actionable errors if metadata is absent.
15. Update all call sites and helpers (`generate_system`, tests, array metadata setup, cleanup) to expect the precomputed metadata instead of triggering analysis themselves.
16. Remove now-dead utilities (`FIFORegistry.reset`, `FIFORegistry.clear_for_module`, `ModuleMetadata.reset_for_analysis`, and any `_reset` helpers that only existed for the dumper-managed lifecycle) or simplify them to match the new ownership model, ensuring incremental refresh paths remain explicit in the new helper instead of the registry.

### Validation & Wrap-up

17. Re-run the focused FIFO tests plus other unit suites touched by the refactor to verify the new flow passes.
18. `source setup.sh && make test-all` to ensure no regressions elsewhere.
19. Review the generated Verilog (spot-check key modules) to confirm metadata-driven wiring remains unchanged apart from the entrypoint simplification.
20. Capture the work in `dones/DONE-slim-fifo-analysis-entrypoint.md`, outlining achieved simplifications, residual risks, and follow-up ideas (e.g., extending the pattern to other metadata families).
21. Stage and commit the changes with a meaningful message once pre-commit passes, following the project’s git guidelines.

## Section 3: Open Questions / Coordination Points

1. Should `collect_fifo_metadata` cache results between calls for large designs, or is reconstructing the dicts on each invocation acceptable for now?
2. Do any external consumers rely on `CIRCTDumper.run_fifo_analysis` (e.g., tooling or scripts outside this repo)? If so, what migration path or compatibility shim is required?
3. How will predicate string generation occur without a full dumper? Is extracting a lightweight `dump_predicate_expr` helper sufficient, or do we need a broader refactor of expression dumping utilities?

## Section 4: Risks & Mitigations

1. **Loss of Incremental Guarantees**: Eliminating `_ensure_fifo_metadata` could leave regenerated modules with stale metadata if partial runs are not carefully handled. Mitigation: ensure `collect_fifo_metadata` accepts a `modules` parameter and properly prunes prior entries; add regression tests to guard it.
2. **Predicate Divergence**: Moving predicate formatting out of `CIRCTDumper` might introduce discrepancies. Mitigation: reuse the existing `PredicateStack` logic and add sanity tests comparing predicate strings from old vs new flows.
3. **API Breakage**: Removing `run_fifo_analysis` may break downstream tooling. Mitigation: document the change, provide deprecation stubs if necessary, and update internal call sites comprehensively before removal.
4. **State Initialisation Complexity**: Passing metadata through the constructor alters how other registries (`array_metadata`, external wiring) expect to be initialised. Mitigation: audit constructor side effects, ensure every attribute is still initialised correctly, and cover with smoke tests.

## Section 5: Summary Checklist

- [ ] Docs updated to describe analysis-before-dumper lifecycle (Items 8–10)
- [ ] Tests rewritten to target `collect_fifo_metadata` and new dumper contract (Items 11–12)
- [ ] FIFO analysis helper returns module/FIFO dicts and no longer mutates dumper state (Items 13–15)
- [ ] Redundant dumper APIs removed and codebase migrated (Items 14–16)
- [ ] Full test suite passes after refactor (`make test-all`) (Item 18)
- [ ] Work recorded in `dones/DONE-slim-fifo-analysis-entrypoint.md` and committed per guidelines (Items 20–21)
