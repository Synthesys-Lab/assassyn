# DONE: Builder Parent Context

## 0. Goal
- Align the builder core with the new invariant that every emitted expression records its owning module while simplifying body tracking.

## 1. Action Items
- [x] Reviewed and updated builder documentation before touching code.
- [x] Refactored the builder to remove the body stack, enforce a mandatory module context, and set expression parents when materialising nodes.
- [x] Updated dependent helpers (naming manager, experimental frontend utilities, bind type checks) to respect the new invariants.
- [x] Added regression coverage for the builder context guarantees.
- [x] Ran `source setup.sh && make test-all`.

## 2. Code Changes
- Documentation updates in `python/assassyn/builder/__init__.md` and `python/assassyn/builder.md` describing the new module/parent guarantees and the simplified body handling.
- Core builder refactor in `python/assassyn/builder/__init__.py`: `current_module` now raises when no module is active, `current_body` derives from the module, `_ctx_stack['body']` is removed, and `ir_builder` always sets parents when a module context is present.
- Support updates in `python/assassyn/builder/naming_manager.py`, `python/assassyn/experimental/frontend/factory.py`, and `python/assassyn/experimental/frontend/module.py` to catch the new guard rails.
- Positive bind tests now accept the explicit `RuntimeError`, reflecting the requirement for an active module context.
- Added `python/unit-tests/test_builder_context.py` to assert the new invariants and keep coverage focused on the builder surface.

## 3. Technical Decisions & Insights
- Opted for a strict `current_module` property that raises without context, mirroring `Singleton.peek_builder()` and making missing scopes fail fast. Compatibility call sites that need a softer failure now catch `RuntimeError`.
- `enter_context_of('body', ...)` is treated as a sanity check instead of mutating state; the owning module owns its body list, so divergence indicates a genuine bug and we surface it immediately.
- Tests that exercise `Module.bind` outside module construction were adjusted to accept the new guard rather than silently ignoring missing contexts, preserving their focus on type checking while aligning with the stricter runtime contract.
- Added targeted unit tests instead of relying solely on integration coverage to catch regressions around parent assignment and insert points.

## 4. Suggested Follow-ups
- Consider providing a documented helper (e.g. `builder.try_current_module()`) for advanced use cases that legitimately probe the context without raising, so callers do not need to sprinkle `try/except`.
- Audit other frontend utilities for similar assumptions about optional module contexts to ensure consistent error messages.
- Extend documentation/tutorials with examples that illustrate the requirement to call IR-producing helpers within module build scopes.
