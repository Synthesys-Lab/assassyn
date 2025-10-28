# Remove Block nodes from IR

## Achievements
- Replaced the legacy `Block`/`CondBlock` hierarchy with flat module body lists backed by predicate push/pop intrinsics.
- Updated builder, visitors, code generators, and analysis helpers to operate on the new representation.
- Adjusted documentation and unit tests (`python/unit-tests/ir_dump/test_blocks.py`) to reflect predicate-driven control flow.

## Follow-ups
- Audit remaining TODO docs that discuss Block semantics and prune outdated guidance.
- Consider extracting shared body-render helpers (`_render_module_body`) into a common utility once other backends migrate.
- Add broader regression coverage for mixed predicate + external exposure scenarios to ensure verilog/simulator parity.

## Notes
- Builder context stacks now track `'body'` lists instead of block objects; predicate stacks remain module-scoped and managed by intrinsic emitters.
- Module/string dumps reuse the former brace-formatting logic by interpreting predicate intrinsics inline, keeping IR dumps familiar while eliminating structural blocks.
- External usage analysis now relies on the expression's parent module directly; this simplified several downstream cross-module ownership checks.
