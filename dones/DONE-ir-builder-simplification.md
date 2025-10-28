# Simplify `ir_builder` Decorator

## Achievements
- Removed the unused `node_type` keyword from the decorator implementation, ensuring the runtime wrapper focuses solely on IR injection and location tracking.
- Updated `python/assassyn/builder.md` so the documented API signature matches the simplified decorator.

## Further Improvements
- Audit downstream tooling for any reliance on `_ir_builder_node_type` style metadata to confirm no latent expectations remain.
- Consider documenting recommended usage patterns for `@ir_builder()` versus `@ir_builder` in the builder guide for stylistic consistency.

## Technical Insights
- Verified across the codebase that no call sites supplied `node_type`, allowing the decorator signature to be reduced without behavioral impact.
- Confirmed pre-commit (Rust/Python lint + full pytest suites) succeeds after the change, demonstrating the simplification introduces no regressions.
