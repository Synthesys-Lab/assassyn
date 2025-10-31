## Goal
Consolidate `FIFORegistry.record_push` and `FIFORegistry.record_pop` in `python/assassyn/codegen/verilog/metadata.py` by introducing a single interaction-recording path, updating the documentation to describe the new API, and ensuring the analysis pass and unit tests consume the unified entry point without regressions.

## Plan
1. **Document the intended registry API adjustments up front**
   - Revise `python/assassyn/codegen/verilog/metadata.md` (section “`FIFORegistry`”) to introduce the new canonical helper (e.g., `record_interaction`) and explain that `record_push` / `record_pop` delegate to it for compatibility.
   - Cross-check `python/assassyn/codegen/verilog/fifo_analysis.md` and any other design notes referencing the old pair to confirm wording stays accurate (no stale “two helpers” phrasing).

2. **Review existing regression coverage**
   - Inspect current FIFO metadata unit tests (e.g., `python/unit-tests/codegen/test_fifo_metadata.py`) to confirm they already exercise both push and pop recording through the registry.
   - Note any gaps worth addressing in the future, but defer adding new tests per instructions.

3. **Implement the shared interaction-recording pathway**
   - Modify `python/assassyn/codegen/verilog/metadata.py`:
     - Add the new `record_interaction` method that infers `is_push` from the expression type, constructs the `FIFOInteraction`, forwards it to `FIFOMetadata.record_interaction`, and returns the shared interaction object.
     - Update `record_push` / `record_pop` to delegate to `record_interaction`, keeping their signatures for existing callers.
     - Tighten docstrings/comments where helpful to explain the unified flow.
   - Update `python/assassyn/codegen/verilog/fifo_analysis.py` so the analysis visitor calls the new helper instead of the duplicated pair, ensuring the refactor exercises the merged code path.

4. **Re-run documentation-aware tests to confirm parity**
   - Execute targeted unit tests: `source setup.sh && pytest python/unit-tests/codegen/test_fifo_metadata.py python/unit-tests/codegen/test_fifo_cleanup_metadata.py python/unit-tests/codegen/test_cleanup.py` to cover FIFO metadata consumers.
   - If runtime permits, run the broader regression suite (`source setup.sh && make test-all`) to ensure no downstream Verilog codegen behaviour changed.

5. **Finalize workflow and record outcomes**
   - Stage changes logically (docs/tests first, then implementation) and craft a commit message following `.cursor/rules/git-message.mdc` once pre-commit hooks pass.
   - Capture a summary in `dones/DONE-verilog-merge-fifo-registry-record.md` outlining the refactor and any follow-up ideas observed during testing.

## Summary Checklist
- [x] Documentation updated to describe the unified FIFO registry helper and its compatibility wrappers.
- [x] Existing FIFO metadata tests reviewed for adequate coverage without new additions.
- [x] `FIFORegistry` implementation refactored to route all recordings through a single code path and analysis updated accordingly.
- [x] Targeted (and ideally full) test suites executed with green results, followed by DONE report and compliant commit message.
