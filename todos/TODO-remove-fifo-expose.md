# TODO: Retire FIFO expose registrations from Verilog codegen

## Section 1: Goal

Eliminate the legacy `dumper.expose('fifo'/'fifo_pop', …)` bookkeeping now that FIFO push/pop predicates are tracked in `ModuleMetadata.fifo`, and teach the Verilog cleanup pass (plus the documentation/tests) to rely exclusively on the metadata-driven flow.

## Section 2: Action Items

### Documentation First

1. Re-read the existing design notes for FIFO handling to ensure the implementation plan stays aligned: `python/assassyn/codegen/verilog/_expr/array.md`, `python/assassyn/codegen/verilog/cleanup.md`, `python/assassyn/codegen/verilog/design.md`, and `python/assassyn/codegen/verilog/top.md`.
2. Update those documents so they no longer mention FIFO exposure via `dumper._exposes`; instead, describe the metadata-driven push/pop wiring path and highlight the new invariants (e.g. `_exposes` is array/expression-only while FIFO logic consumes `ModuleMetadata.fifo`).

### Tests Before Features

3. Extend `python/unit-tests/codegen/test_fifo_metadata.py` to assert that, after visiting a module with FIFO traffic, `dumper._exposes` contains no FIFO keys while `module_metadata.fifo` still records the expected predicates (this should fail until the implementation drops the exposes).
4. Add a new regression (e.g. `python/unit-tests/codegen/test_fifo_cleanup_metadata.py`) that generates a simple module with both push and pop operations, runs Verilog generation through `CIRCTDumper`, and checks the emitted cleanup code for the expected `*_push_valid`, `*_push_data`, and `*_pop_ready` signals derived solely from metadata predicates.
5. Re-run the existing FIFO integration regression (`python/unit-tests/codegen/test_fifo_pop_metadata.py`) in the updated test harness to capture any assumptions about expose-driven wiring up front.

### Implementation

6. Modify `python/assassyn/codegen/verilog/_expr/array.py`:
   - Remove the `dumper.expose('fifo', …)` and `dumper.expose('fifo_pop', …)` calls.
   - Ensure the metadata recording (`metadata.fifo.record_push/pop`) remains intact and, if necessary, add any defensive initialisation so every module with FIFO ops has a populated `ModuleMetadata` entry before cleanup runs.
7. Refactor `python/assassyn/codegen/verilog/cleanup.py` to pivot FIFO wiring onto metadata:
   - Retrieve `module_metadata = dumper.module_metadata[dumper.current_module]` early in `cleanup_post_generation`.
   - Build grouped views of `module_metadata.fifo.pushes`/`pops` keyed by the FIFO port to reproduce the existing valid/data/ready logic (preserving predicate reductions and muxing behaviour), but sourcing predicates/data directly from the metadata entries.
   - Remove or guard the existing `_exposes` branch that handled `Port` keys so it no longer runs; keep the array/expr exposure handling untouched.
8. Audit remaining code for assumptions about `dumper.expose('fifo*')` (search for `'fifo_pop'` / `'fifo'` expose calls) and adjust any leftover references so they rely on metadata instead.
9. Ensure auxiliary helpers (e.g. any convenience functions in `design.py` or `module.py`) still create `ModuleMetadata` instances before metadata is consumed, adding assertions or initialisation if required.

### Validation & Wrap-up

10. Run the updated unit tests in the recommended order: `source setup.sh && pytest python/unit-tests/codegen/test_fifo_metadata.py python/unit-tests/codegen/test_fifo_cleanup_metadata.py python/unit-tests/codegen/test_fifo_pop_metadata.py`.
11. Follow up with the broader regression suite `source setup.sh && make test-all` to confirm no collateral regressions.
12. Document the work in `dones/DONE-remove-fifo-expose.md`, summarising the behavioural changes and any follow-up ideas uncovered.
13. Stage and commit all changes with a guideline-compliant message (after sourcing the environment so the pre-commit hook runs).

## Section 3: Open Questions / Coordination Points

1. Confirm whether any downstream tooling still inspects `_exposes` for FIFO information; if so, note the consumer so we can migrate it during implementation.
2. Verify that removing the FIFO exposes does not break any externally consumed hooks (e.g. debugging utilities) and update the plan if new stakeholders emerge during the audit.

## Section 4: Risks & Mitigations

1. **Predicate Formatting Drift**: Ensure predicate strings pulled from metadata already include the necessary parentheses; if not, add normalisation helpers while refactoring cleanup.
2. **Mux Reconstruction Accuracy**: When port metadata contains multiple pushes from the same module, replicate the existing mux-building semantics (including type casts) to avoid synthesising different hardware.
3. **Test Fixture Fragility**: The new cleanup-focused unit test will need stable anchors in the generated code; prefer regexes or helper functions that tolerate harmless formatting changes.
