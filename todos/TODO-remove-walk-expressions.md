# TODO: Remove `walk_expressions` helpers from `CIRCTDumper`

## Section 1: Goal

Eliminate the now-redundant `CIRCTDumper._walk_expressions` / `walk_expressions` helpers in `python/assassyn/codegen/verilog/design.py` since module bodies are already flattened (`dones/DONE-remove-block.md`). Update all call sites and documents to iterate module bodies directly, ensuring expression-driven analyses keep the same behavior and ordering.

## Section 2: Action Items

### Analysis & Preparation

1. Re-read `dones/DONE-remove-block.md` and skim `docs/design/` notes on flattened bodies to reconfirm the invariants (module bodies are ordered lists of `Expr` instances plus intrinsics).
2. Inventory every code and documentation reference with `rg "_walk_expressions"` / `rg "walk_expressions"` so the migration plan covers `system.py`, `design.md`, `system.md`, `top.md`, and any lingering helpers.
3. Inspect `python/assassyn/codegen/verilog/design.py` to confirm no subclasses or external utilities rely on the helper methods being part of the public dumper surface; note any comments that must be rewritten once the helpers vanish.

### Documentation (update before code)

4. Revise `python/assassyn/codegen/verilog/design.md` to remove the `_walk_expressions` section and describe the direct-body iteration pattern; add a short note that predicate intrinsics keep ordering intact per `DONE-remove-block`.
5. Update `python/assassyn/codegen/verilog/system.md` to explain that system-level scans iterate `module.body` entries directly (filtering `Expr` instances) when collecting external outputs and async calls.
6. Adjust any other docs that mention the helpers—specifically `python/assassyn/codegen/verilog/top.md` and cross-references under `docs/design/`—so guidance aligns with the new approach; run `rg "_walk_expressions"` afterward to ensure no stale documentation remains.

### Testing (prepare before implementation)

7. Identify the smallest pytest targets that exercise system generation (e.g., `python/unit-tests` suites covering Verilog dumping or async call handling). Plan to run them once code changes are in place to guard against regressions in cross-module exposure logic.
8. Budget time for a follow-up `source setup.sh && make test-all` run after implementation since helper removal touches shared code paths across the generator.

### Implementation

9. Remove `_walk_expressions` and `walk_expressions` definitions from `CIRCTDumper` in `design.py`, making sure no residual imports or class attributes depend on them.
10. Update `python/assassyn/codegen/verilog/system.py`:
    - Replace the two helper-driven loops with direct iteration over `module.body`, filtering for `Expr` instances.
    - Keep existing guard rails (`body is None`, `ModuleBase` ownership checks) and preserve iteration order.
    - Add any missing imports (`Expr`) if the file does not already expose the symbol.
11. After edits, run `rg "_walk_expressions"` and `rg "walk_expressions"` to confirm no code references remain; address any surprises (e.g., dead tests or comments) discovered by the search.

### Validation & Wrap-up

12. Execute targeted pytest command(s) from step 7 and ensure they pass.
13. Run `source setup.sh && make test-all` to verify the broader regression matrix still succeeds after helper removal.
14. Summarize the refactor in a new `dones/DONE-remove-walk-expressions.md`, capturing the rationale, testing performed, and follow-up recommendations per documentation policy.

### Git Workflow

15. Stage changes in logical chunks (docs → implementation → summary) and commit with meaningful messages; allow pre-commit hooks to run under the project environment (`source setup.sh` beforehand).

## Section 3: Open Questions

1. Do any downstream tools or external scripts import `walk_expressions` dynamically (e.g., via `getattr`) that need deprecation notes or compatibility shims?
2. Should we introduce a shared utility for “iterate expressions from a module body” to centralize the `isinstance(expr, Expr)` filter now that multiple modules perform it?

## Section 4: Potential Follow-ups

1. Audit other code generators (e.g., simulator backends) for similar helper patterns that can be simplified post-block removal.
2. Expand documentation with a short “module body traversal” best-practice snippet so future contributors default to direct iteration instead of recreating dumper helpers.

## Section 5: Completion Checklist

- [x] Documentation updated (`design.md`, `system.md`, `top.md`, related references)
- [x] Helper functions removed and call sites migrated
- [x] Targeted pytest suite passing
- [x] `make test-all` passing
- [x] Summary recorded in `dones/DONE-remove-walk-expressions.md`

## Section 6: Summary

- Removed the dumper-specific expression walkers and refreshed documentation to emphasize direct iteration over flattened module bodies.
- Updated system traversal logic to filter `Expr` instances inline, keeping external output and async call analyses intact while simplifying the API surface.
- Validated changes with focused FIFO metadata pytest coverage and the full `make test-all` regression suite.

## Section 7: Completion Log

- Documentation edits: `python/assassyn/codegen/verilog/design.md`, `system.md`, `top.md`.
- Implementation updates: dropped `_walk_expressions` helpers in `python/assassyn/codegen/verilog/design.py` and migrated loops in `python/assassyn/codegen/verilog/system.py`.
- Tests executed: `source setup.sh && python -m pytest python/unit-tests/codegen/test_fifo_pop_metadata.py`; `source setup.sh && make test-all` (all suites passing).
