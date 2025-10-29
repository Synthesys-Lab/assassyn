# TODO: Introduce a dedicated FIFO metadata pre-pass

## Section 1: Goal

Refactor the Verilog codegen pipeline so FIFO interaction metadata (push/pop operations, predicates, per-module port usage) is collected in a dedicated analysis pass before any Verilog-generation visits occur. The runtime `FIFORegistry.clear_for_module` bookkeeping becomes unnecessary because the registry will be populated once per compile run, ahead of code emission.

## Section 2: Action Items

### Foundational Analysis

1. Re-read the existing documentation describing metadata responsibilities: `python/assassyn/codegen/verilog/metadata.md`, `cleanup.md`, `design.md`, `module.md`, `top.md`, and `docs/design/internal/pipeline.md` to enumerate every consumer of FIFO interaction data and the invariants they expect (ordering, predicate fidelity, deduplicated module interaction lists).
2. Catalog current metadata collection points (primarily `_expr/array.py`’s FIFO handlers) and trace how predicate context (`dumper.get_pred()`, condition stack contents) and module scoping feed into the registry/module views. Identify any additional context (e.g., async call stack, wait-until state) that a pre-pass must either replicate or capture beforehand.
3. Inspect existing unit/regression tests touching FIFO metadata (`python/unit-tests/codegen/test_fifo_metadata.py`, `test_fifo_cleanup_metadata.py`, `test_fifo_pop_metadata.py`) to understand coverage gaps that need updates once the pre-pass is introduced.
4. Evaluate the current predicate-stack management in `CIRCTDumper` (`cond_stack`, `push_condition`, `wait_until` handling) and determine how to extract this logic into reusable helpers so the pre-pass can share the exact same behaviour.

### Proposed Design Draft

5. Sketch the architecture for a pre-pass visitor (`FIFOAnalysisVisitor` or an extension of an existing IR visitor) that traverses modules without emitting code. Document how it will:
   - Initialize per-module metadata containers before the Verilog-emission phase.
   - Evaluate the same predicate stack semantics as the main dumper (handling `push_condition`, `pop_condition`, `wait_until`, etc.).
   - Record FIFO interactions (module, port, expression, predicate) in a central registry without relying on codegen-time side effects.
6. Define the lifecycle coordination between the new pre-pass and `CIRCTDumper`:
   - When and how the analysis visitor runs relative to `visit_system` / `visit_module`.
   - How results get injected into or shared with the existing `FIFORegistry` / `ModuleMetadata` structures (e.g., reusing current dataclasses versus introducing pre-pass-specific storage then freezing it).
7. Decide whether the pre-pass runs at the system level only or also supports single-module regeneration. Outline fallback strategies for incremental emits (e.g., running the pre-pass selectively for the touched module) and how pre-pass results are cached or refreshed.
8. Plan the data flow for predicate contexts: ensure the extracted helper utilities can be consumed identically by both visitors (pre-pass and codegen) to prevent divergence.
9. Evaluate handling of lazy constructs (e.g., async calls referencing other modules) and confirm the pre-pass won’t miss dependencies because of order of traversal.

### Documentation First

10. Update `python/assassyn/codegen/verilog/metadata.md` to describe the new two-phase architecture (analysis pass then code emission), clarifying ownership of FIFOs and how module metadata is seeded before codegen.
11. Refresh other docs (`cleanup.md`, `design.md`, `module.md`, `top.md`) to remove references to codegen-time FIFO logging and replace them with the pre-pass narrative.
12. Document the new visitor/intermediate structures (e.g., under `docs/design/internal/pipeline.md`) so future contributors understand the rationale and sequence.

### Tests Before Implementation

13. Draft new or expanded unit tests that fail under the current implementation but will pass after the pre-pass is introduced:
   - Ensure a module revisited after the pre-pass still sees fresh metadata without requiring `clear_for_module`.
   - Confirm cross-module FIFO interactions are recorded once during the analysis pass and remain stable across subsequent codegen calls.
   - Validate predicate stacks in scenarios with nested `push_condition`/`pop_condition` are captured identically by the pre-pass (compare old vs new metadata in tests).
14. Build regression coverage for incremental workflows: simulate running the pre-pass for a single module and validate downstream codegen emits consistent Verilog.
15. Run the targeted pytest suite to capture failure modes before implementation and note expected diffs.

### Implementation

16. Implement the FIFO analysis visitor:
   - Leverage shared utilities for predicate management.
   - Populate shared `FIFORegistry`/`ModuleMetadata` instances (or intermediate placeholders) without emitting Verilog code.
   - Ensure the visitor respects module scoping and handles all FIFO-related expressions encountered in the IR.
17. Extract predicate-stack handling logic from `CIRCTDumper` into reusable helpers or a mixin consumed by both the pre-pass visitor and the dumper, ensuring identical semantics.
18. Modify `CIRCTDumper` to consume pre-populated metadata:
    - Skip in-situ FIFO recording during `_expr/array.py` (`codegen_fifo_push/pop`); ensure they now rely solely on the pre-pass data.
    - Remove `clear_for_module` invocations and any other state resets that are obsolete with the pre-pass.
19. Update expression handlers to reference the pre-pass data (e.g., confirm they still return the expressions for eventual Verilog wiring but do not mutate metadata).
20. Adjust cleanup/top-generation helpers to operate unchanged on the newly sourced metadata; ensure helper signatures remain compatible.
21. Delete obsolete code paths (e.g., module-level FIFO registers no longer needed for logging) and clean up registry APIs accordingly.
22. Ensure the pre-pass integrates cleanly with incremental workflows: provide APIs for analyzing a single module and caching/updating metadata accordingly before invoking codegen.

### Validation & Wrap-up

23. Re-run the enriched FIFO-focused tests and ensure they now pass under the pre-pass workflow.
24. Execute `make test-all` (after `source setup.sh`) to verify global regressions.
25. If feasible, compare generated Verilog for representative modules before and after the refactor to ensure functional parity (possibly with existing golden tests).
26. Document the completed work in `dones/DONE-fifo-prepass-metadata.md`, highlighting the new phase, benefits, and any follow-ups (e.g., potential reuse of the analysis visitor for other metadata).
27. Stage and commit the documentation, tests, and code changes following the project’s git standards, ensuring pre-commit hooks pass.

## Section 3: Open Questions / Coordination Points

1. Can the predicate stack logic be factored cleanly so both the pre-pass and codegen share a single implementation without duplicating subtle behaviour?
2. How should incremental workflows trigger the pre-pass? Do we need API hooks for “analyze module X only” versus re-analyzing the entire system?
3. Will the pre-pass need visibility into other registries (e.g., array metadata) to accurately reproduce context for codegen-time decisions?
4. Are there downstream consumers outside Verilog codegen that rely on metadata being populated lazily during code emission? If so, plan an adaptation strategy.

## Section 4: Risks & Mitigations

1. **Divergent Predicate Semantics**: If the pre-pass implements predicate stacking differently than codegen, generated metadata may mismatch actual emission. Mitigation: factor predicate management into shared helpers and add regression tests comparing results.
2. **Incremental Build Complexity**: Introducing a mandatory pre-pass might complicate single-module regeneration workflows. Mitigation: design the visitor to support module-scoped runs and cache results to avoid redundant work.
3. **Performance Regression**: Running an extra traversal could impact build times. Mitigation: benchmark with representative systems and optimize the visitor (avoid expensive recomputation, reuse IR traversal results).
4. **API Changes**: Removing runtime metadata mutations could break existing utilities. Mitigation: audit all public interfaces, provide adapters or deprecations, and update the docs/tests in lock-step.

## Section 5: Summary Checklist

- [x] Predicate management helpers extracted for shared use (Items 4 & 17)
- [x] Documentation updated to describe the pre-pass architecture (Items 10–12)
- [x] Tests prepared to enforce pre-pass behaviour before implementation (Items 13–15)
- [x] Analysis visitor and codegen refactor implemented with incremental support (Items 16–22)
- [x] Test suites (focused + `make test-all`) verified under the new workflow (Items 23–24)
- [x] Work captured in `dones/` and staged for commit per guidelines (Items 25–27)
