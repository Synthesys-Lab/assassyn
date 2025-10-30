# TODO Add meta_cond metadata to ArrayWrite, FIFOPop, and AsyncCall

## Goal
Align array write, FIFO pop, and async call semantics with the existing `Log.meta_cond` contract so that both simulator and Verilog backends receive explicit predicate metadata without having to re-traverse condition stacks.

## Plan

1. **Re-read design references and current behaviour**
   - Revisit `python/assassyn/ir/expr/array.md`, `python/assassyn/ir/expr/expr.md`, and `python/assassyn/ir/expr/call.md` to confirm current IR contracts and how `FIFOPop`/`AsyncCall` lifecycles work.
   - Review `docs/design/lang/intrinsics.md`, `docs/design/internal/pipeline.md`, and `docs/design/arch/memory.md` to understand how predicate gating and multi-writer coordination are supposed to work.
   - Walk through Verilog metadata collection in `python/assassyn/codegen/verilog/fifo_analysis.py` and `python/assassyn/codegen/verilog/metadata.py`, plus simulator helpers in `_expr/array.py` and `_expr/call.py`, to document the exact touch points that currently reconstruct predicates from the stack.

2. **Document the intended metadata contract before coding**
   - Update the IR docs (`python/assassyn/ir/expr/array.md`, `python/assassyn/ir/expr/expr.md`, `python/assassyn/ir/expr/call.md`) to describe the new `meta_cond` attribute, including its type (`Bits(1)`), default capture via `get_pred()`, and how existing helpers automatically populate it.
   - Extend the high-level design note that already mentions `Log.meta_cond` (likely `docs/design/lang/intrinsics.md` or a new subsection in `docs/design/internal/pipeline.md`) to explain that array writes, FIFO pops, and async calls now share the same metadata path for backend gating.

3. **Design tests up front**
   - Sketch a new IR dump regression (probably in `python/unit-tests/ir_dump/`) that builds a module with nested `if`/`else` blocks and exercises: an array write, a FIFO push, a FIFO pop, and an async call under different predicates. The expected dump should assert the `meta_cond` captured value (e.g., references to guard signals) similar to existing `test_log_meta_cond_metadata`.
   - Plan simulator/verilog-facing checks: either extend an existing simulator codegen smoke test or add a targeted unit test to assert that generated Rust/Verilog snippets conditionally gate events using the recorded metadata (for example by inspecting emitted strings for an `if (<meta>)` guard).

4. **Implement IR-side metadata capture**
   - Adjust `ArrayWrite.__init__` to accept and store a trailing `meta_cond` operand (defaulted to `None` so legacy construction paths can still instantiate the node). Mirror the `Log` accessor pattern by adding a `meta_cond` property and updating `__repr__` to include the predicate comment only when present.
   - Update every construction path to provide the metadata automatically:
     * In `python/assassyn/ir/expr/writeport.py::create_write`, call `get_pred()` before instantiating `ArrayWrite`.
     * Audit any other direct `ArrayWrite` constructors (e.g., tests or specialized builders) and patch them to pass the metadata explicitly.
   - Apply the same pattern to `FIFOPush`, `FIFOPop`, and `AsyncCall`:
     * Extend their constructors to append `meta_cond`.
     * Capture `get_pred()` inside `Port.push()`, `Port.pop()`, and `Bind.async_called()` so IR authors do not need to pass the metadata manually.
     * Add convenience accessors and update `__repr__` strings with optional predicate comments for debugging parity.

5. **Propagate metadata through backends**
   - Verilog: refactor `FIFOAnalysisVisitor` and the `FIFORegistry` interactions to pull the predicate directly from `expr.meta_cond` without re-formatting the condition stack—matching the FIFO push/pop IR operands instead of rebuilding strings. Maintain a gentle fallback/warning path if metadata is unexpectedly absent.
   - Simulator Python emitters:
     * Update `_expr/array.codegen_array_write`, `_expr/call.codegen_fifo_push`, `_expr/call.codegen_fifo_pop`, and `_expr/call.codegen_async_call` to thread the `meta_cond` into the generated Rust glue (e.g., wrap event pushes behind predicate guards or record the predicate alongside the event payload).
     * Ensure `python/assassyn/codegen/simulator/simulator.py` and `tools/rust-sim-runtime/src/runtime/xeq.rs` evolve in lockstep so that new metadata fields (like an optional predicate) compile and behave correctly.
   - Audit any other consumers (namer, cleanup passes, metadata summaries) to make sure the extra operand does not break tuple unpacking or assumptions about operand counts.

6. **Execute and validate**
   - Run focused unit tests (`python -m pytest python/unit-tests/ir_dump/test_array_ops.py::test_array_ops_dump ...`) to ensure array writes, FIFO pushes/pops, and async calls all capture `meta_cond` as expected before kicking off `source setup.sh && make test-all`.
   - Inspect generated simulator/Verilog artefacts for the new gating to confirm behaviour matches the documented contract.

7. **Polish and wrap up**
   - Produce a `dones/DONE-arraywrite-fifo-meta-cond.md` summary covering achievements, follow-up ideas (e.g., migrate pushes, finish removing predicate stack duplication), and any nuanced design decisions.
   - Stage changes, craft a commit message that follows `.cursor/rules/git-message.mdc`, and rerun `pre-commit` via `git commit --amend --no-edit` if needed after sourcing `setup.sh`.

---

## Status

- [x] Reviewed the relevant docs and current implementations for ArrayWrite, FIFO push/pop, and AsyncCall.
- [x] Updated IR and design documentation to describe the new `meta_cond` contract.
- [x] Added regression tests covering predicate metadata capture across array writes, FIFO interactions, and async calls.
- [x] Implemented implicit `meta_cond` propagation in IR constructors and builder helpers.
- [x] Threaded metadata through simulator and Verilog backends, simplifying predicate handling and updating consumers.
- [x] Ran focused unit tests plus the full `python/unit-tests` and `python/ci-tests` suites.
- [x] Prepared DONE notes, ready to stage code changes, and align commit with project guidelines.
