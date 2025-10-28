# Module Stack Refactor

## Achievements
- Replaced the legacy `_ctx_stack` dictionary in `SysBuilder` with a single `_module_stack`, matching the flat IR model and eliminating empty body frames.
- Simplified the module context API (`enter_context_of`/`exit_context_of`) to operate purely on modules while retaining predicate-balance assertions.
- Updated builder documentation (`python/assassyn/builder.md`, `python/assassyn/builder/__init__.md`) and combinational wrapper usage to reflect the streamlined interface.

## Follow-ups
- Audit remaining DONE/TODO notes that reference `'body'` context handling and update or remove those to avoid confusion.
- Consider a quick pass over downstream tooling to ensure no latent expectations for block contexts remain (e.g., code generators or analysis helpers).
- Evaluate whether additional invariants (like explicit module open/close tracing) should be logged for debugging once more modules adopt the flat flow.

## Notes
- Added explicit guards that reject `enter_context_of(None)` or modules without an initialised `body`, reducing the chance of silent context mismatches.
- `make test-all` passes after the refactor, covering both builder-centric and downstream regression suites.
