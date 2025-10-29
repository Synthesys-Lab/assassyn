# TODO: Verilog FIFO Metadata Symmetry

## Goal or Issue
- Rename the module-level metadata dataclass `PostDesignGeneration` to `ModuleMetadata` across code and docs so the name reflects its broader responsibilities.
- Introduce a dedicated `FIFOMetadata` structure that records FIFO push/pop occurrences, including which module triggered them, a reference to the expression, and the predicate active at emission time using `CIRCTDumper.get_pred()`.
- Keep documentation, tests, and downstream consumers aligned with the new metadata model to avoid regressions during Verilog code generation.

## Action Items
1. **Background refresh**
   - Re-read `python/assassyn/codegen/verilog/metadata.py` and `python/assassyn/codegen/verilog/metadata.md` to catalogue existing fields, the current `PostDesignGeneration` responsibilities, and how metadata is consumed.
   - Skim `python/assassyn/codegen/verilog/design.py` to trace `module_metadata` usage (initialisation, `get_pred`, downstream access) and pinpoint touch points for the upcoming rename and FIFO tracking changes.
   - Review FIFO codegen helpers in `python/assassyn/codegen/verilog/_expr/array.py` and their documentation to understand how pushes/pops are currently appended and where predicate context is available.
   - Inspect any tests or utilities referencing FIFO metadata (search for `module_metadata` and `pushes`/`pops`) to gauge current coverage.

2. **Documentation updates before implementation**
   - Update `python/assassyn/codegen/verilog/metadata.md` to rename the class references to `ModuleMetadata`, describe the new `FIFOMetadata` dataclass, and explain how predicate strings and module associations are recorded.
   - Adjust `python/assassyn/codegen/verilog/design.md` (and any other docs referencing `PostDesignGeneration`) to reflect the new naming and to mention how `module_metadata` now exposes FIFO predicate data.
   - Note any additional documentation (e.g., tutorials or internal design notes) that cite the old class name or FIFO tracking semantics and align them with the new structure.

3. **Test planning (TDD-first)**
   - Locate existing unit tests touching `module_metadata` (likely in `python/unit-tests/codegen`); determine gaps related to predicate capture or FIFO metadata shape.
   - Sketch a new test module (e.g., `python/unit-tests/codegen/test_fifo_metadata.py`) that:
     - Builds a minimal module with FIFO push/pop operations under different conditional stacks.
     - Confirms that `CIRCTDumper.module_metadata[module].fifo.pushes/pops` (or the new accessor) expose both module references and predicates recorded via `get_pred()`.
     - Verifies backward compatibility for existing consumers (e.g., `has_finish`, `calls`) remains intact.
   - Prepare fixtures or helper builders to construct predicate contexts without heavy system scaffolding.

4. **Test implementation (expected initial failure)**
   - Author the new/updated tests reflecting the desired metadata API and predicate expectations.
   - Run the targeted test suite via `source setup.sh && pytest python/unit-tests/codegen/test_fifo_metadata.py`, expecting failures because the implementation is not yet updated.
   - Stage and (if policy allows) create an intermediate failing-test commit using `--no-verify` should the hooks block due to the intentional failure.

5. **Code implementation**
   - Rename the dataclass in `python/assassyn/codegen/verilog/metadata.py` to `ModuleMetadata`, ensuring imports (`design.py`, other codegen modules) and typing aliases follow suit.
   - Define a `FIFOMetadata` dataclass capturing push/pop details, including:
     - `producer_module: Module`
     - `expr: FIFOPush` / `FIFOPop` references (or separate lists for each)
     - `predicate: str` captured by invoking `CIRCTDumper.get_pred()` when the operation occurs.
   - Refactor `CIRCTDumper.module_metadata` initialisation in `design.py` to instantiate the updated structure and expose an appropriate container (e.g., `module_metadata[module].fifo.pushes.append(...)`).
   - Modify `_expr/array.py` FIFO helpers to construct/update `FIFOMetadata` entries, pulling the predicate via `dumper.get_pred()` instead of only appending raw expressions.
   - Ensure `TYPE_CHECKING` sections and runtime imports avoid circular dependencies; update `metadata.py` typing hints accordingly.
   - Propagate the rename through all code references (`from .metadata import PostDesignGeneration` → `ModuleMetadata`, etc.) using `rg` to avoid missing spots.

6. **Align existing consumers**
   - Audit all downstream uses of `module_metadata` (e.g., `top.py`, other codegen stages) to update attribute access patterns and ensure they leverage the richer FIFO metadata where appropriate.
   - Where consumers only require expressions, adapt them to handle the new structure (e.g., iterating over metadata entries and accessing `.expr`).
   - Add brief code comments in high-complexity areas clarifying how predicate strings are to be consumed.

7. **Verification**
   - Re-run the focused FIFO metadata test and any existing suites that rely on module metadata: `source setup.sh && pytest python/unit-tests/codegen/test_fifo_metadata.py` and other relevant files (e.g., `test_fifo_pop_metadata.py`).
   - Execute a broader regression sweep (`source setup.sh && make test-all`) if runtime permits to confirm no unintended regressions surfaced.

8. **Documentation and design consistency check**
   - Revisit updated docs to ensure terminology, field lists, and cross-references match the final code implementation.
   - Confirm doc examples (if any) reflect the new predicate-tracking API.

9. **Release workflow**
   - Stage changes incrementally, creating commits that respect the repo’s commit message policy (`.cursor/rules/git-message.mdc`) and run pre-commit hooks (`source setup.sh` before `git commit`).
   - Document the completed work in `dones/` (e.g., `dones/DONE-verilog-fifo-metadata.md`) summarising implementation details, non-obvious decisions, and follow-on ideas.
   - Leave the workspace clean, ready for subsequent tasks.

## Risks & Mitigations
- **Predicate stack edge cases**: Use tests covering nested conditionals to ensure `get_pred()` outputs align with expectations; add assertions or helper functions if necessary.
- **Legacy consumer breakage**: Carefully refactor each `module_metadata` consumer and run targeted tests to ensure they handle the new structure; consider temporary compatibility accessors if a large refactor is required.
- **Circular imports**: Keep heavy imports inside `TYPE_CHECKING` guards and rely on forward references in dataclasses to prevent runtime cycles.

## Summary Checklist
- [x] Background study of metadata usage and FIFO helpers completed.
- [x] Documentation (`metadata.md`, `design.md`, related notes) updated to reflect `ModuleMetadata` + `FIFOMetadata`.
- [x] New FIFO metadata unit tests authored and initially failing to capture desired behaviour.
- [x] Code updated to rename the dataclass, introduce `FIFOMetadata`, and record predicates via `get_pred()`.
- [x] Downstream consumers adjusted and tests passing (`pytest` + `make test-all`).
- [x] DONE report drafted with insights and follow-up suggestions, repo left clean.

## Completion Notes
- Added `FIFOMetadata` with push/pop entries storing module references and predicates, plus backward-compatible `pushes`/`pops` accessors.
- Updated documentation and all metadata consumers to the new API before landing code changes.
- Introduced `test_fifo_metadata` to exercise predicate tracking and confirmed regressions via targeted pytest runs and `make test-all`.
