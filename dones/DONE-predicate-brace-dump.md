# DONE: Predicate brace/comment dump in IR representation

## Goal
Render predicate push/pop intrinsics as visually bracketed regions in IR dumps while making it explicit they are not structural if-statements.

## Actions Completed
- Updated docs: `python/assassyn/ir/block.md` explains the special dump rule.
- Updated tests: `python/unit-tests/ir_dump/test_blocks.py` asserts `// PUSH_CONDITION`, `} // POP_CONDITION`, and `if ` appear.
- Implemented dumping: `python/assassyn/ir/block.py` renders
  - `if cond { // PUSH_CONDITION` on push, increases indent
  - `} // POP_CONDITION` on pop, decreases indent first

## Changes
- Docs
  - Clarified formatting-only nature of the new markers, tied to `Singleton.repr_ident`.
- Tests
  - Shifted from looking for raw intrinsic names to brace/comment markers.
- Code
  - Centralized formatting in `Block.__repr__` to avoid touching intrinsic `__repr__` and to manage indentation consistently.

## Non-obvious decisions and insights
- Chose block-level dump customization to avoid modifying intrinsic semantics and to keep formatting concerns localized.
- Indentation balance relies on existing builder invariants (balanced push/pop); no extra guards added here to avoid redundant state.
- `CondBlock.__repr__` left unchanged to preserve legacy behavior; future work could unify visual styles if desired.

## Future Improvements
- Consider a dedicated pretty-printer layer to decouple dump logic from IR nodes.
- Add optional flags to toggle this formatting for debugging vs. raw intrinsic dump.
- Explore merging predicate display with `CondBlock` style for consistent visuals across conditional constructs.
