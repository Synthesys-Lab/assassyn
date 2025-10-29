# TODO: Encapsulate Array Payload Helper Interfaces

## Section 1: Goal

Introduce a first-class helper on `python/assassyn/ir/array.Array` so callers can query `array.is_payload(SRAM)` / `array.is_payload(DRAM)` instead of repeating identity checks against `memory._payload`. The refactor should preserve the simplified ownership semantics documented in `dones/DONE-array-owner-simplification.md` while consolidating payload detection across the IR, code generators, and simulator.

## Section 2: Action Items

### Analysis & Preparation

1. **Re-confirm ownership invariants**
   - Re-read `python/assassyn/ir/array.md`, `docs/design/internal/array-ownership.md`, and the SRAM/DRAM module docs (`python/assassyn/ir/memory/{base,sram,dram}.md`) to restate the assumptions around `Array.owner`, `_payload`, and auxiliary registers.
   - Inspect `dones/DONE-array-owner-simplification.md` to ensure the new helper does not contradict the recent simplification decisions.
2. **Inventory current payload checks**
   - Search for direct usages of `owner._payload` and `array is memory._payload` across the codebase (`python/assassyn/codegen/verilog/{array.py,_expr/array.py,cleanup.py,system.py,top.py,metadata.py,utils.py}`, `python/assassyn/codegen/simulator/simulator.py`, and related tests) and note how each call site obtains the memory reference (class vs. instance) to inform the helper signature.
   - Confirm there are no other memory subclasses (e.g., future caches) relying on bespoke checks that the helper might need to support.

### Documentation (draft before code)

3. **Update Array documentation**
   - Extend `python/assassyn/ir/array.md` (Array class section) with the new `is_payload` method, documenting accepted arguments (`SRAM`, `DRAM` classes) and the identity semantics the helper enforces.
   - Provide a short usage snippet demonstrating `array.is_payload(SRAM)` returning `True` only when `array.owner` is an `SRAM` instance and the array is that instance's `_payload`.
4. **Refresh ownership design notes**
   - Amend `docs/design/internal/array-ownership.md` to reference the helper as the canonical way to detect payload buffers, replacing text that currently instructs readers to manually check `array is memory._payload`.
   - Update memory module docs (`python/assassyn/ir/memory/base.md` and `python/assassyn/ir/memory/sram.md`) so guidance aligns with the new helper rather than direct identity comparisons.

### Testing (write before implementation)

5. **Extend ownership unit tests**
   - Augment `python/unit-tests/test_array_owner.py` with cases that fail under current behaviour:
     - Positive checks: `sram._payload.is_payload(SRAM)` and `dram._payload.is_payload(DRAM)` should return `True`.
     - Negative checks: `sram.dout.is_payload(SRAM)` and `sram._payload.is_payload(DRAM)` should return `False`, along with a plain `RegArray` returning `False` for both memory types.
   - Run the new tests to confirm they fail prior to implementation.

### Implementation

6. **Add the helper to Array**
   - Implement `Array.is_payload` in `python/assassyn/ir/array.py`, accepting either a `MemoryBase` subclass (e.g., `SRAM`, `DRAM`) or an instance; validate the argument type, then check whether `self.owner` is an instance of the supplied class and `self is owner._payload`.
   - Include an explicit docstring, type hints, and guard rails (raise `TypeError` for unsupported inputs) to keep misuse discoverable.
7. **Adopt the helper across call sites**
   - Replace ad-hoc payload checks with the new method in Verilog codegen (`python/assassyn/codegen/verilog/{array.py,_expr/array.py,cleanup.py,system.py,top.py,metadata.py,utils.py}`) and the simulator (`python/assassyn/codegen/simulator/simulator.py`).
   - Keep accesses to `_payload` confined to the helper where possible; where the owning instance is already available, prefer `array.is_payload(type(owner))` to avoid duplicating identity logic.
   - Update any inline comments to reference the helper instead of direct identity comparisons.

### Validation & Wrap-up

8. **Run targeted regressions**
   - Execute `source setup.sh && python -m pytest python/unit-tests/test_array_owner.py` to validate the new helper.
   - Run focused CI suites touching SRAM/DRAM behaviour: `source setup.sh && python -m pytest python/ci-tests/test_sram.py python/ci-tests/test_dram.py`.
9. **Full regression sweep**
   - Finish with `source setup.sh && make test-all` to ensure no broader regressions in codegen or simulator paths.
10. **Document outcomes**
    - Capture the implementation summary, insights, and follow-up ideas in `dones/DONE-array-payload-helper.md`, referencing updated docs and tests.
11. **Commit workflow**
    - Stage changes in logical chunks (docs → tests → code) to enable intermediate checks if needed.
    - Commit with a descriptive message following the guideline, e.g., `[IR] Add Array payload helper for SRAM/DRAM`.

## Section 3: Open Questions

1. Should `Array.is_payload` accept only memory classes (as required) or also instances to accommodate call sites that already hold the concrete memory object?
2. Do any non-memory owners (future `MemoryBase` subclasses, external integrations) need comparable helpers, and how should the API extend to support them?

## Section 4: Potential Follow-ups

1. Provide complementary helpers on `MemoryBase` (e.g., `memory.payload_array`) to further reduce direct `_payload` exposure.
2. Audit for other repeated ownership checks (e.g., `dout` detection) that could benefit from similar encapsulation to simplify downstream code.

## Section 5: Completion Checklist

- [x] Ownership docs updated to recommend `Array.is_payload`.
- [x] Unit tests cover helper behaviour for SRAM, DRAM, and negative cases.
- [x] All direct payload identity checks replaced with the helper.
- [x] Regression test suites pass after sourcing `setup.sh`.
- [x] Summary logged in `dones/DONE-array-payload-helper.md`.

---

## Summary Checklist (2025-02-14)

0. **Goal Achieved**
   - Introduced a first-class `Array.is_payload` helper and migrated documentation, tests, and code generation to rely on it while keeping ownership semantics unchanged.
1. **Action Items Completed**
   - [x] Re-confirmed ownership invariants across IR docs and DONE notes.
   - [x] Inventoried existing payload checks in IR, simulator, and Verilog backends.
   - [x] Drafted documentation updates for `Array`, ownership design notes, and memory modules before implementation.
   - [x] Added failing unit test coverage for SRAM/DRAM payload detection prior to the helper implementation.
   - [x] Implemented `Array.is_payload` with class/instance support and refactored all call sites to use it.
   - [x] Executed targeted (`python/unit-tests/test_array_owner.py`, `python/ci-tests/test_sram.py`, `python/ci-tests/test_dram.py`) and full (`make test-all`) regressions after sourcing `setup.sh`.
   - [x] Logged the outcome in `dones/DONE-array-payload-helper.md`.
2. **Codebase Changes**
   - Added `Array.is_payload` with argument validation and identity checks confined to the helper (`python/assassyn/ir/array.py`), plus corresponding documentation in `python/assassyn/ir/array.md`.
   - Updated simulator and Verilog generation paths to rely on the helper instead of direct `_payload` identity comparisons (`python/assassyn/codegen/verilog/{array.py,_expr/array.py,cleanup.py,system.py,top.py}`, `python/assassyn/codegen/simulator/simulator.py`) and refreshed backend docs.
   - Extended ownership unit tests to cover positive/negative SRAM/DRAM payload detection (`python/unit-tests/test_array_owner.py`) and synchronized ownership/memory documentation (`docs/design/internal/{array-ownership.md,changelog.md,pipeline.md,simulator.md}`, `python/assassyn/ir/memory/{base.md,dram.md,sram.md}`).
   - Verified all suites via `python -m pytest python/unit-tests/test_array_owner.py`, `python -m pytest python/ci-tests/test_sram.py python/ci-tests/test_dram.py`, and `make test-all`.
3. **Technical Decisions**
   - Accepted both memory classes and instances in `is_payload` to accommodate existing call patterns while rejecting unsupported inputs with a `TypeError` for early feedback.
   - Used `getattr(instance, '_payload', None)` to keep the helper resilient to future memory expansions that may rename internals while still documenting `_payload` as the canonical attribute.
   - Standardised documentation messaging so downstream contributors adopt the helper, minimizing future drift between code and design notes.
