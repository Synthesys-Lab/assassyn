Predicate intrinsics and simulator-first tests

0. Goal achieved

- Introduced predicate intrinsics to support condition scoping without relying on block structure: `PUSH_CONDITION`, `POP_CONDITION` (uppercase opcodes) and a pure frontend helper `get_pred()`.
- Implemented a builder-managed predicate stack and wired codegen for simulator and Verilog (minimal). Added two simulator-first tests to validate behavior.

1. Checklist of action items completed

- Added builder `cond` stack and synchronized it when entering/exiting `CondBlock` contexts.
- Added `PUSH_CONDITION`/`POP_CONDITION` opcodes and frontend APIs `push_condition(cond)` and `pop_condition()`.
- Added `get_pred()` frontend helper which ANDs all active conditions on the builder predicate stack.
- Simulator codegen: emit `if (...) {` on `PUSH_CONDITION` and close `}` on `POP_CONDITION` in `python/assassyn/codegen/simulator/modules.py`.
- Verilog dumper: update `cond_stack` on `PUSH_CONDITION`/`POP_CONDITION` and use `get_pred()` for exposure/gating (no direct `if` emission).
- Tests:
  - `python/ci-tests/test_async_call_pred.py` implements async-call using `push_condition`/`pop_condition` (simulator only).
  - `python/ci-tests/test_pred_nested.py` validates nested `get_pred()` semantics with `Driver` entry and bit-slice even check (simulator only).
- Documentation: extended `docs/design/lang/intrinsics.md` to include predicate intrinsics and usage patterns.

2. Changes made (by area)

- Builder
  - `python/assassyn/builder/__init__.py`: added `cond` stack to `_ctx_stack`, push/pop around `CondBlock`, maintained caches.

- IR / Frontend
  - `python/assassyn/ir/expr/intrinsic.py`:
    - Added opcodes `PUSH_CONDITION` (914) and `POP_CONDITION` (915).
    - Added APIs `push_condition(cond)`, `pop_condition()`; mirrored builder predicate stack for immediate frontend semantics.
    - Added `get_pred()` pure helper; returns `Bits(1)(1)` if stack empty, else AND of all conditions.
  - `python/assassyn/ir/expr/__init__.py`, `python/assassyn/frontend.py`: exposed the new APIs.
  - `python/assassyn/ir/expr/comm.py`: added `and_all` helper (iterable variant).

- Simulator codegen
  - `python/assassyn/codegen/simulator/modules.py`: handled `PUSH_CONDITION`/`POP_CONDITION` directly in the visitor to build real nested blocks with indentation.
  - `python/assassyn/codegen/simulator/_expr/intrinsics.py`: noted that predicate intrinsics are handled by the module visitor, not as inline expressions.

- Verilog codegen
  - `python/assassyn/codegen/verilog/_expr/intrinsics.py`: push/pop `cond_stack`; used `get_pred()` for finish/exposure signals.
  - `python/assassyn/codegen/verilog/design.py`: kept `get_pred()` returning `Bits(1)(1)` on empty stack (PyCDE expression compatibility). Verified simple async-call case passes; testbench gating remains separate.

- Tests (simulator-only for this phase)
  - `python/ci-tests/test_async_call_pred.py`: async-call parity using predicate intrinsics; added `verilog=False`.
  - `python/ci-tests/test_pred_nested.py`: nested conditions; uses `Driver` entry and `(cnt[0][0:0]) == Bits(1)(0)`; added `verilog=False`.

- Documentation
  - `docs/design/lang/intrinsics.md`: added sections for `push_condition`/`pop_condition` and `get_pred()`, with examples and notes on simulator/Verilog handling.

3. Non-obvious technical decisions

- Builder stack mirroring in APIs: The predicate stack is updated both by entering/exiting `CondBlock` and by calling `push_condition`/`pop_condition`. This ensures consistent `get_pred()` behavior regardless of whether conditions are expressed with `with Condition(cond):` or via explicit intrinsics.
- Simulator handling at the visitor layer: For `PUSH_CONDITION`/`POP_CONDITION`, emitting explicit `if { ... }` with indentation in the module visitor (instead of the intrinsic codegen table) preserves block structure and keeps performance by skipping code when the predicate is false.
- Verilog `get_pred()` baseline literal: For PyCDE code, `get_pred()` needs a typed literal; we kept `Bits(1)(1)` in design emission (PyCDE world) while the testbench has its own gating rules. The testbench definition is currently independent and was not unified in this phase.
- Test strategy: Simulator-first to validate semantics early; Verilog enablement deferred. The simple async-call Verilog probe was used to validate baseline behavior and surfaced the `Bits` import and predicate literal differences in testbench, which were resolved within the testbench generation rule (kept out of this change’s scope for broader unification).

4. Follow-ups and suggestions

- Unify predicate handling between design and testbench layers to avoid literal mismatches and import assumptions; make `get_pred()` consistently consumable in both contexts.
- Add lints/checks for balanced `PUSH_CONDITION`/`POP_CONDITION` usage and unmatched stacks.
- Migrate `Condition` frontend API to emit predicate intrinsics transparently, then deprecate/remove `Block`/`CondBlock` and simplify visitor traversal.
- Re-enable Verilog for the new predicate tests after generalizing gating and exposure (avoid referencing non-exposed locals in tb conditions).

5. Files touched (high-signal)

- `docs/design/lang/intrinsics.md`
- `python/assassyn/builder/__init__.py`
- `python/assassyn/ir/expr/intrinsic.py`
- `python/assassyn/ir/expr/__init__.py`, `python/assassyn/frontend.py`
- `python/assassyn/ir/expr/comm.py`
- `python/assassyn/codegen/simulator/modules.py`, `python/assassyn/codegen/simulator/_expr/intrinsics.py`
- `python/assassyn/codegen/verilog/_expr/intrinsics.py`, `python/assassyn/codegen/verilog/design.py`
- `python/ci-tests/test_async_call_pred.py`, `python/ci-tests/test_pred_nested.py`



