# TODO: Verilog Array Expression Type Contracts

## Goal or Issue
- Ensure the Verilog array expression helpers (`codegen_array_read/write` and FIFO helpers) declare their CIRCT entry-point contract explicitly by typing `dumper` as `CIRCTDumper`.
- Guard the helpers with `@enforce_type` so incorrect callsites fail fast with a `TypeError`, while keeping existing behaviour for valid invocations.
- Keep documentation, design notes, and tests in sync with the stricter runtime typing policy.

## Action Items
1. **Context refresh**
   - Re-read `python/assassyn/codegen/verilog/_expr/array.py` alongside its documentation sibling `python/assassyn/codegen/verilog/_expr/array.md` to confirm current control flow and exposed metadata.
   - Skim the `CIRCTDumper` API in `python/assassyn/codegen/verilog/design.py` to confirm which attributes the array helpers rely on, so the type annotation aligns with the real contract.
   - Review `python/assassyn/utils/enforce_type.py` and `docs/design/internal/enforce_type.md` for any constraints (e.g., forward-reference limitations) that might affect decorating these helpers.

2. **Documentation updates before code**
   - Update `python/assassyn/codegen/verilog/_expr/array.md` to describe the new `CIRCTDumper`-typed `dumper` parameter and the runtime type enforcement expectations for each public helper.
   - Cross-link the documentation to the enforce-type design note (`docs/design/internal/enforce_type.md`) so future readers know these helpers participate in the runtime contract.
   - Check whether `_expr/__init__.md` or other cross-referenced docs need a brief note that array expression dispatch now demands a `CIRCTDumper`.

3. **Test design (TDD preference)**
   - Audit existing tests in `python/unit-tests/codegen` (especially `test_fifo_pop_metadata.py`) to confirm which helpers are already exercised indirectly.
   - Add a focused test module (e.g., `python/unit-tests/codegen/test_array_codegen_type_enforcement.py`) that:
     - Imports the array helper functions.
     - Verifies each helper raises `TypeError` when given an invalid `dumper` or `expr`, demonstrating the decorator is active.
     - Optionally sanity-checks that calling with a properly typed stub (e.g., a lightweight `CIRCTDumper` fixture or a subclass with the minimal attributes touched by the helper) still succeeds for the happy path.
   - Run the new test in isolation (`source setup.sh && pytest python/unit-tests/codegen/test_array_codegen_type_enforcement.py`) expecting it to fail before implementation, then stage and commit the failing test with `--no-verify` if necessary.

4. **Implementation work**
   - Add `from __future__ import annotations` to `python/assassyn/codegen/verilog/_expr/array.py` so type hints can reference `CIRCTDumper` without runtime imports.
   - Introduce a `TYPE_CHECKING` guarded import (`from ..design import CIRCTDumper`) to avoid circular dependencies while keeping editors aware of the type.
   - Import `enforce_type` from `python/assassyn/utils/enforce_type` and decorate every helper in this module.
   - Annotate each `dumper` parameter as `CIRCTDumper` and ensure return annotations remain accurate; add explicit `return None` where the helper currently relies on implicit `None` to keep the control-flow clear under stricter typing.
   - Double-check the helper bodies for any callsites that might now need adjustments (e.g., ensuring the `expr` types already imported match the decorator expectations).

5. **Regression verification**
   - Re-run the new enforcement-focused test plus existing Verilog codegen suites:
     - `source setup.sh && pytest python/unit-tests/codegen/test_array_codegen_type_enforcement.py`
     - `source setup.sh && pytest python/unit-tests/codegen/test_fifo_pop_metadata.py`
     - `source setup.sh && pytest python/unit-tests/test_array_type_enforcement.py` (sanity-check that broader array typing logic remains intact).
   - If runtime allows, finish with `source setup.sh && make test-all` to ensure no hidden regressions.

6. **Documentation alignment check**
   - Re-read the updated docs alongside the final code to confirm terminology, function signatures, and behavioural descriptions match exactly.
   - Ensure any new cross-references resolve correctly (`mkdocs`/markdown lint expectations).

7. **Version-control workflow**
   - After documenting and introducing the failing test, create an intermediate commit capturing the spec (`git commit -am "docs,test: codify verilog array type contract"` using `--no-verify` only if tests are intentionally red).
   - Once implementation is complete and tests pass, stage all changes and commit with a message following `.cursor/rules/git-message.mdc` (e.g., `feat(verilog): enforce dumper type on array expr helpers`) while running pre-commit hooks (`source setup.sh` beforehand).
   - Wrap up by drafting `dones/DONE-verilog-array-type-enforcement.md` summarising the work, follow the act-on-todo checklist, and leave the workspace clean for hand-off.

## Risks & Mitigations
- **Potential circular import**: Use `TYPE_CHECKING` and postponed evaluation of annotations to avoid runtime import loops.
- **Decorator side-effects on callsites**: Run the broader Verilog codegen tests to surface any call sequence that previously passed non-`CIRCTDumper` stubs (fix callsites if encountered).
- **Test fixture complexity**: Prefer lightweight fixtures or partial stubs so the new tests remain fast; fall back to real `SysBuilder` generation only if absolutely required.

## Summary Checklist
- [x] Validated existing array expression docs, dumper contract, and type-enforcement design notes.
- [x] Updated `python/assassyn/codegen/verilog/_expr/array.md` with CIRCTDumper typing details and enforce-type cross references.
- [x] Added enforcement-focused regression tests in `python/unit-tests/codegen/test_array_codegen_type_enforcement.py`.
- [x] Decorated array helpers with `@enforce_type`, typed dumper as `CIRCTDumper`, and resolved runtime binding.
- [x] Ran targeted pytest suites plus `make test-all` to confirm no regressions.
- [x] Re-reviewed docs against code to ensure terminology and links align.
- [x] Prepared DONE report and readied changes for commit per workflow.
