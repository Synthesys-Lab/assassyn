# TODO: Retire CIRCTDumper dependency from Array Metadata Registry

## Section 1: Goal

Remove the `CIRCTDumper` argument from `ArrayMetadataRegistry.collect` now that module bodies are already flattened (`dones/DONE-remove-block.md`). The registry should inspect module bodies directly, keeping read/write port enumeration unchanged while updating documentation, tests, and callers to reflect the simplified API.

## Section 2: Action Items

### Analysis & Preparation

1. Re-read `dones/DONE-remove-block.md` and inspect `CIRCTDumper._walk_expressions` to confirm flattened bodies guarantee expression nodes are top-level `Expr` instances, ensuring a direct iteration over `module.body` preserves ordering and coverage.
2. Use `rg "ArrayMetadataRegistry.collect"` and `rg "walk_expressions"` to inventory all Python and documentation references (e.g., `system.py`, `design.md`, `metadata.md`, unit tests) so the refactor touches every dependency exactly once.

### Documentation (update before code)

3. Revise `python/assassyn/codegen/verilog/array.md` to document the new `collect(sys)` signature, explain that module bodies are walked directly, and cite `dones/DONE-remove-block.md` as the rationale for dropping the dumper dependency.
4. Update `python/assassyn/codegen/verilog/metadata.md` so the “When Metadata is Populated” section no longer references `CIRCTDumper._walk_expressions`, instead noting that the registry iterates `module.body` entries.
5. Audit `python/assassyn/codegen/verilog/design.md`, `system.md`, and related docs (`module.md`, `top.md`) to ensure descriptions of metadata collection and `_walk_expressions` remain accurate—clarify that `_walk_expressions` now only services other analyses, not the array registry.

### Testing (prepare before implementation)

6. Adjust `python/unit-tests/test_array_owner.py` (and any other test using `collect`) to invoke the new API (`dumper.array_metadata.collect(sys)` or `ArrayMetadataRegistry().collect(sys)`), adding an assertion that the registry functions without a dumper. Run the test to observe the expected failure until the implementation is updated.

### Implementation

7. Update `python/assassyn/codegen/verilog/array.py`:
   - Drop the `dumper` parameter and TYPE_CHECKING import of `CIRCTDumper`.
   - Iterate modules via their `body` lists, filtering `Expr` instances to find `ArrayRead`/`ArrayWrite` nodes.
   - Keep writer/read-port registration logic intact and maintain deterministic ordering.
   - Refresh the method docstring and any inline comments to reflect the new traversal approach.
8. Modify `python/assassyn/codegen/verilog/system.py` to call `dumper.array_metadata.collect(sys)` and remove any now-unused local variables or imports. Re-run `rg` afterward to ensure no other callers rely on the old signature.
9. Clean up ancillary fallout: remove stale type hints or helper aliases tied to the old API (e.g., if `ArrayMetadataRegistry.collect` was re-exported elsewhere) and ensure `CIRCTDumper` initialisation still satisfies type checkers.

### Validation & Wrap-up

10. Execute targeted unit coverage: `source setup.sh && python -m pytest python/unit-tests/test_array_owner.py` (plus any other suite that depends on array metadata once identified during analysis).
11. Run the broader regression pass: `source setup.sh && make test-all` to confirm backends and docs stay healthy.
12. Leave a summary in `dones/DONE-remove-array-metadata-dumper.md` capturing the refactor outcomes, follow-ups, and technical insights per the documentation policy.

### Git Workflow

13. Stage and commit in logical chunks (docs → tests → implementation), using meaningful commit messages and allowing pre-commit hooks to run without `--no-verify`.

## Section 3: Open Questions

1. Do any ancillary utilities instantiate `ArrayMetadataRegistry` outside the dumper (e.g., simulator helpers) that require additional API adjustments?
2. Is there value in extracting a shared helper for “iterate expressions from a module body” to avoid duplicating the `Expr` filtering logic elsewhere?
3. Should `_walk_expressions` in `CIRCTDumper` be deprecated entirely once remaining call sites migrate, or does it still serve other analyses?

## Section 4: Potential Follow-ups

1. Evaluate other components still relying on `_walk_expressions` and migrate them to metadata-driven or direct body iteration where possible.
2. Add regression tests that exercise metadata collection with downstream modules to ensure body traversal covers both `sys.modules` and `sys.downstreams`.
3. Document a concise “module body iteration” helper in the design docs to guide future contributors away from dumper dependencies.

## Section 5: Completion Checklist

- [x] Documentation updated (`array.md`, `metadata.md`, `design.md`, `system.md`, related references)
- [x] Tests adjusted for the new API and passing
- [x] `ArrayMetadataRegistry.collect` refactored with no dumper dependency
- [ ] Regression suites executed (`python -m pytest ...`, `make test-all`)
- [x] Summary recorded in `dones/DONE-remove-array-metadata-dumper.md`

## Section 6: Summary

- All action items have been executed; regression suite completion awaits a full `make test-all` run due to timeout.

## Section 7: Completion Log

- Documentation, tests, and implementation updated to remove the dumper dependency from `ArrayMetadataRegistry.collect`.
- `python -m pytest python/unit-tests/test_array_owner.py` passes; `make test-all` timed out after 159s while running, so completion is pending further confirmation.
- Summary recorded in `dones/DONE-remove-array-metadata-dumper.md`.
