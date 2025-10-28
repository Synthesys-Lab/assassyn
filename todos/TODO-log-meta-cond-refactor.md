# TODO: Consolidate Verilog log predicate handling

## Goal
Document and implement the shift to `Log.meta_cond`-driven predicate exposure in the Verilog backend, replacing ad-hoc condition traversal for log statements.

## Plan
1. **Refresh documentation**: Update the relevant design or developer docs to describe how `Log.meta_cond` governs Verilog trace gating so reviewers understand the intended behaviour before code changes.
2. **Review existing coverage**: Revisit `dones/DONE-replace-condition-with-intrinsics.md`, then inspect all test cases referenced by `@ci-tests` with `verilog=False` enabled to confirm the current expectations before touching code.
3. **Implement the refactor**: Modify `python/assassyn/codegen/verilog/design.py` and `python/assassyn/codegen/verilog/_expr/intrinsics.py::codegen_log` to consume only `Log.meta_cond` for predicate exposure while still exposing any dynamic predicate expression once.
4. **Adjust supporting expectations**: Update or add assertions so the existing tests pass without relying on the old exposure behaviour.
5. **Run validation**: Execute `source setup.sh && make test-all` (or an equivalent targeted subset first, if faster) and confirm the suite passes.
6. **Document the outcome**: Capture the work and insights in a new `dones/DONE-*.md`, then stage and commit following the project’s git message guidelines.

## Notes
- Keep intermediate commits logical (e.g. documentation refresh, refactor, clean-up) and run `pre-commit` hooks prior to the final commit.
- Reuse the metadata handling patterns described in `dones/DONE-log-predicate-metadata.md` and `dones/DONE-log-accessors.md` to avoid reintroducing redundant condition exposure.
- When reviewing the `@ci-tests` configuration, pay special attention to log predicate behaviour under `verilog=False` to ensure the refactor maintains simulator parity.

## Completion Checklist
- [x] Refreshed design documentation to describe `Log.meta_cond`-driven gating in Verilog.
- [x] Reviewed `dones/DONE-replace-condition-with-intrinsics.md` alongside all `@ci-tests` entries that run with `verilog=False`.
- [x] Refactored exposure logic in `python/assassyn/codegen/verilog/design.py` and `python/assassyn/codegen/verilog/_expr/intrinsics.py::codegen_log` to rely on `Log.meta_cond`.
- [x] Verified the existing simulator-focused tests remain valid without additional cases.
- [x] Ran `source setup.sh && make test-all` to confirm the suite passes post-refactor.
- [x] Documented the work in `dones/DONE-log-meta-cond-refactor.md`.
