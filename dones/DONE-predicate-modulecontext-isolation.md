# Predicate stack isolation with ModuleContext

0. Goal achieved

- Isolate the builder-managed predicate stack per module using a ModuleContext record on the module stack.
- Make both Condition blocks and predicate intrinsics (push_condition/pop_condition) consistently manipulate the same per-module stack.

1. Checklist of action items completed

- Introduced ModuleContext { module, cond_stack } and pushed onto the module stack on module entry.
- Added SysBuilder helpers: get_predicate_stack, push_predicate, pop_predicate.
- Wired CondBlock enter/exit to push/pop predicates via helpers; assert empty cond_stack on module exit.
- Updated intrinsics: push_condition/pop_condition call builder helpers; get_pred() reads current ModuleContext stack.
- Removed the deprecated CURRENT_CYCLE alias; unified on current_cycle().
- Documentation updated to reflect ModuleContext and per-module predicate semantics.
- Added an isolation test scaffold to exercise multi-module predication separation.

2. Changes made (by area)

- Builder
  - `python/assassyn/builder/__init__.py`: Added ModuleContext; refactored module stack to hold contexts; implemented per-module predicate helpers; updated enter/exit logic; enforced predicate stack empty on module exit.
  - `python/assassyn/builder/__init__.md`: New documentation describing SysBuilder, ModuleContext, predicate helpers, and invariants.

- IR / Frontend
  - `python/assassyn/ir/expr/intrinsic.py`: push_condition/pop_condition now use builder helpers; get_pred() reads builder.get_predicate_stack(); removed CURRENT_CYCLE alias; kept current_cycle().
  - `python/assassyn/ir/block.py`: Cycle helper explicitly uses current_cycle().

- Codegen
  - `python/assassyn/codegen/verilog/_expr/intrinsics.py` and simulator counterpart: comment updates to refer to current_cycle(). No behavior change.

- Docs
  - `docs/design/lang/intrinsics.md`: Document per-module predicate stack via ModuleContext; standardized on current_cycle().
  - `python/assassyn/ir/block.md`: Updated narrative to reference current_cycle() and predicate semantics.

- Tests
  - `python/ci-tests/test_pred_multi_module_isolation.py`: Introduced isolation test scaffolding (Driver/Testbench-based logging as needed). See notes.

3. Non-obvious technical decisions

- Predicate stack as module-context state: Using an array-of-structs (ModuleContext) on the module stack avoids global maps, ties lifetime to module scope, and prevents cross-module leakage by construction.
- Dual entry points: Predicates can be introduced by both CondBlock and explicit intrinsics; centralizing push/pop in SysBuilder guarantees stack coherence regardless of expression style.
- Assertive exits: Asserting empty predicate stack on module exit quickly surfaces imbalances, preventing subtle leaks into subsequent modules.
- Alias removal: Consolidating on current_cycle() reduces API surface and avoids confusion in testbenches and codegen.

4. Follow-ups and suggestions

- Convert the isolation test to assert via Driver-visible signals, since module-local logs may not propagate in all test modes.
- Add lint to detect unmatched push_condition/pop_condition usage across traces.
- Consider enum for Block kinds to improve readability and type safety.
