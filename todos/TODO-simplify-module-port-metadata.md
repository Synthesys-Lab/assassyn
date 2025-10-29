# TODO: Simplify Module Port Metadata Inference

## Context
- `python/assassyn/codegen/verilog/design.py` currently computes `is_downstream`, `is_sram`, `is_driver`, and the `pushes`/`calls`/`pops` lists before delegating to `generate_module_ports`.
- These values can be derived inside `generate_module_ports` from the module type, dumper state (e.g., `async_callees`), and the existing `module_metadata` lookup.
- The goal is to consolidate the inference at the helper level, reduce redundant state in `visit_module`, and keep documentation/tests aligned with the streamlined interface.

## Plan
1. **Document the intended refactor before coding**
   - Review `python/assassyn/codegen/verilog/design.md`, `python/assassyn/codegen/verilog/module.md`, and `python/assassyn/codegen/verilog/README.md` sections describing `visit_module` and `generate_module_ports`.
   - Update these docs to explain that downstream/SRAM/driver detection and metadata extraction now happen inside `generate_module_ports`, including the rationale for removing redundant flag plumbing.
   - Stage the documentation edits for the initial commit so design intent and reasoning are captured ahead of code changes.

2. **Prepare or adjust regression coverage**
   - Inspect existing Verilog codegen tests under `python/assassyn/test` (and any golden artifacts) that assert generated port lists or FIFO handshakes.
   - If coverage is lacking, add a focused unit test that exercises a module with pushes, pops, and async calls, asserting that port emission reflects the recorded metadata. Since this is a refactor, the new test should already pass but will guard against regressions once the helper is simplified.
   - Run the targeted test (or suite) to capture the pre-refactor baseline before touching the implementation.

3. **Refactor module port generation logic**
   - Update `generate_module_ports` to derive downstream/SRAM/driver status internally and to fetch `pushes`/`calls`/`pops` from `dumper.module_metadata.get(node)` with safe fallbacks.
   - Simplify `visit_module` to stop computing the redundant locals and pass only the node (and optional metadata object) to the helper; adjust signatures, type hints, and imports accordingly.
   - Ensure handshake generation remains identical by validating set construction/orderings and by updating inline comments or docstrings to match the new data flow.
   - Touch any additional call sites or helpers affected by the signature change and remove now-unused state.

4. **Validation and wrap-up**
   - Execute the focused regression(s) plus `source setup.sh && make test-all` to ensure the refactor preserves behavior.
   - Split commits per guideline: documentation first, then tests and code; make sure pre-commit hooks succeed.
   - Record the outcome and potential follow-ups in `dones/DONE-simplify-module-port-metadata.md` after the refactor lands.

## Risks & Mitigations
- Missing metadata entries for a module could cause attribute errors; mitigate by defaulting to empty metadata containers or by asserting initialization during `visit_module`.
- Hidden callers of `generate_module_ports` might rely on the old signature; mitigate by repo-wide search/updates and by syncing docs/tests in the same change set.
- Port ordering changes could invalidate goldens; mitigate by preserving iteration order and adjusting tests only if semantically necessary.

## Section 5: Completion Checklist
- [x] Documentation updated to describe metadata-driven inference within `generate_module_ports`.
- [x] Regression coverage verifies port emission still reflects pushes/calls/pops after the refactor.
- [x] `visit_module` delegates without redundant flag/list computation.
- [x] `source setup.sh && make test-all` passes after changes.

## Section 6: Summary
- Centralizing flag and metadata inference inside `generate_module_ports` eliminates duplication in `visit_module` while keeping module interfaces stable.
- The work will start with documentation updates, reinforce tests to guard expected behavior, refactor the helper and caller, and finish with full-suite validation and a `dones/` summary.

## Completion Summary
- [x] Updated `design.md`, `module.md`, and `README.md` to document metadata-driven inference inside `generate_module_ports`.
- [x] Reused existing metadata-focused regression (`test_fifo_pop_metadata.py`) and confirmed baseline plus header import coverage.
- [x] Refactored `generate_module_ports` to compute roles/metadata internally and simplified `visit_module` accordingly.
- [x] Validated with `source setup.sh && pytest python/unit-tests/codegen/test_fifo_pop_metadata.py python/unit-tests/codegen/test_pycde_header_import.py` and `source setup.sh && make test-all`.
