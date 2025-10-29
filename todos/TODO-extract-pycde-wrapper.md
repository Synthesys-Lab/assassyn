# TODO: Extract PyCDE Parameter Modules into a Runtime Wrapper

## Context
- `python/assassyn/codegen/verilog/utils.py` currently embeds two `@modparams` definitions (`FIFO`, `TriggerCounter`) directly in the Verilog header that gets spliced into generated code.
- We want these reusable parameterized modules to live in a Python-side runtime helper (new `python/assassyn/pycde_wrapper.py`) and have generated code import them instead of redefining inline.
- Need to preserve existing functionality and update documentation/tests to reflect the new structure.

## Plan
1. **Document intent before coding**
   - Survey design docs under `docs/design/` and backend/runtime notes to identify where the PyCDE support is described.
   - Draft updates outlining that shared PyCDE modules (like `FIFO`, `TriggerCounter`) now live in `assassyn.pycde_wrapper` and are imported by generated headers.
   - Stage these doc edits for the first commit so the rationale precedes implementation.

2. **Define/adjust tests ahead of implementation**
   - Inspect existing tests for Verilog code generation (`python/assassyn/test` and related golden outputs) to find coverage for header emission.
   - Add or modify an automated check that asserts the header string references `assassyn.pycde_wrapper` imports rather than inline class definitions (e.g., snapshot difference or string expectation).
   - Ensure the new/updated test initially fails because the runtime wrapper is not yet in place.

3. **Implement runtime wrapper extraction**
   - Create `python/assassyn/pycde_wrapper.py` exporting the two `@modparams` classes with identical behavior to the current inline definitions; add any necessary packaging hook in `python/assassyn/__init__.py` for user access.
   - Refactor `python/assassyn/codegen/verilog/utils.py` to remove the hardcoded class definitions from `HEADER` and instead import them (`from assassyn.pycde_wrapper import FIFO, TriggerCounter`) inside the emitted header string.
   - Update code-generation logic if needed to ensure the generated modules still instantiate these helpers correctly.

4. **Validation and wrap-up**
   - Run `source setup.sh && make test-all` to confirm tests (including the new header check) pass with the refactor.
   - Split commits: first for documentation changes, second for tests + runtime/code updates; ensure final commit passes pre-commit hooks.
   - Capture the outcome and follow-up ideas in `dones/` (`DONE-pycde-wrapper.md`) after implementation.

## Risks & Mitigations
- **Generated code breakage**: Verify that the emitted import path matches runtime packaging; add an integration test if necessary.
- **Packaging visibility**: Ensure `assassyn.pycde_wrapper` is accessible when users consume generated modules (consider adding to `__all__`).
- **Test brittleness**: Prefer resilient assertions (e.g., regex or targeted substring checks) so formatting adjustments don't cause false negatives.

## Section 5: Completion Checklist

- [x] Updated Verilog pipeline and utility docs to reference the shared runtime wrapper.
- [x] Added a unit test that enforces `design.py` imports `FIFO` and `TriggerCounter` from `assassyn.pycde_wrapper`.
- [x] Introduced `python/assassyn/pycde_wrapper.py` and refactored `HEADER` to import the shared helpers.
- [x] Ran `source setup.sh && make test-all` after refactor to confirm regressions are green.

## Section 6: Summary

- Documented the new wrapper flow across design docs and utility references so future updates keep code and docs aligned.
- Strengthened regression coverage with `python/unit-tests/codegen/test_pycde_header_import.py`, catching inadvertent header regressions.
- Added `python/assassyn/pycde_wrapper.py` and updated `python/assassyn/codegen/verilog/utils.py` so generated designs reuse the shared FIFO/TriggerCounter implementations.
- Verified the refactor with the full `make test-all` suite sourced from `setup.sh`, ensuring the codegen pipeline remains stable.
