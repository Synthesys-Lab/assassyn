# DONE: Remove CycledBlock and Block.CYCLE; unify on conditional blocks

## Goal
Remove `CycledBlock` and `Block.CYCLE`, migrate to `CondBlock`-only control flow with cycle gating expressed via `CURRENT_CYCLE` conditions (e.g., `with Cycle(N): ...`).

## Actions Completed
- Updated `python/assassyn/ir/block.md` to remove `CycledBlock`/`Block.CYCLE` and document `Cycle()` returning `CondBlock`.
- Updated codegen docs:
  - `python/assassyn/codegen/verilog/_expr/intrinsics.md`: reflect CURRENT_CYCLE-based condition handling.
  - `python/assassyn/codegen/simulator/modules.md`: remove `CycledBlock` mentions.
- Adjusted unit test docstring `python/unit-tests/ir_dump/test_blocks.py` to drop `CycledBlock` wording.
- Removed `CycledBlock` class and `Block.CYCLE` in `python/assassyn/ir/block.py`.
- Removed `CycledBlock` handling in codegen:
  - `codegen/verilog/design.py`: only handles `CondBlock` on condition stack.
  - `codegen/verilog/_expr/intrinsics.py`: eliminate `CycledBlock` import/branch; translate `self.cycle_count` occurrences in predicates to DUT path.
  - `codegen/simulator/modules.py`: drop `CycledBlock` branch; rely on `CondBlock` (CURRENT_CYCLE lowers via intrinsics).
- Verified repository contains no live references to `CycledBlock` or `Block.CYCLE` except in historical docs/todos.
- Ran `python/ci-tests/test_driver.py` successfully; simulator and Verilator runs passed.

## Changes and Rationale
- Simplified control flow: one block kind (`CONDITIONAL`) reduces indirection and aligns with write-good-code.
- `Cycle()` remains as a concise helper constructing `CondBlock(current_cycle() == UInt(64)(n))`.
- Verilog testbench paths translate `cycle_count` to `dut.global_cycle_count.value` consistently.

## Follow-ups
- Consider converting block kind integers to an Enum for clarity.
- Sweep remaining documentation in `todos/` and historical DONE docs when convenient.
- Add a small unit test asserting `isinstance(Cycle(5), CondBlock)` if coverage is desired.

## Notable Decisions
- Kept behavior stable by only removing the redundant node/constant; no functional change to `Cycle()`.
- Minimal, low-invasion edits focused on IR and codegen boundaries.

