# DONE: Unify Predicate-Driven Value Selection Helpers

0. **Goal Achieved**  
   Unified the predicate/value mux construction used by cleanup array writes and FIFO pushes behind a shared helper while preserving generated Verilog semantics.

1. **Action Items Completed**  
   - [x] Surveyed existing cleanup logic and relevant design documents.  
   - [x] Added regression tests capturing array write and FIFO push renderings.  
   - [x] Introduced `_emit_predicate_mux_chain` and refactored cleanup to use it for arrays and FIFOs.  
   - [x] Updated documentation to describe the shared helper and its usage.  
   - [x] Verified no parallel mux patterns exist in `module.py` or `system.py`.  
   - [x] Ran `source setup.sh && make test-all`.

2. **Code Changes & Improvements**  
   - Added `python/unit-tests/codegen/test_cleanup.py` to lock in mux/predicate output ordering for multi-writer arrays and FIFO pushes.  
   - Replaced the ad-hoc mux builders in `python/assassyn/codegen/verilog/cleanup.py` with `_emit_predicate_mux_chain` plus `_format_reduce_or`, removing the legacy `build_mux_chain` indirection.  
   - Updated `cleanup.md`, `design.md`, and `docs/design/internal/pipeline.md` to reference the shared helper and clarify that cleanup now funnels arrays/FIFOs through it.  
   - Helper design keeps aggregation formatting customizable so future consumers can supply alternate defaults without copying logic.  
   - Potential future improvement: promote `_emit_predicate_mux_chain` to a small utilities module if additional backends start needing the same pattern.

3. **Non-Obvious Technical Decisions**  
   - The helper accepts render callbacks plus a reduction formatter to match the subtle differences between array predicates (`Bits` casting) and FIFO predicates (explicit `(expr)` wrapping with a default literal). This avoided hard-coding policy into the helper while keeping call sites concise.  
   - Tests derive signal name prefixes from metadata instead of hard-coding literals so they remain stable even if the naming manager changes, while still asserting the exact mux structure.  
   - Confirmed no other modules emulate the same predicate/value folding, so scoping the helper within `cleanup.py` maintains low surface area for now.
