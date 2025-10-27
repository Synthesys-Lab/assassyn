# Fix Array Read Cache with Predicate-Associated Invalidation

## 0. Goal achieved / current state

Re-enabled array read caching in `Array.__getitem__` with predicate-aware invalidation by associating each predicate frame with its own cache dictionary. When a predicate is pushed, array reads are cached in that frame's dictionary. When popped, the cache is automatically discarded, preventing reads created under that predicate from leaking into subsequent code. This fixes the issue where array reads were incorrectly reused across predicate scopes, which broke FSM and other conditional tests after the predicate intrinsics refactor.

## 1. Checklist of action items completed

- [x] Re-enabled array read caching in `python/assassyn/ir/array.py::__getitem__`
- [x] Added `PredicateFrame` class to bundle condition value and array-read cache dict
- [x] Updated `push_predicate`/`pop_predicate` to push/pop `PredicateFrame` objects
- [x] Updated `__getitem__` to probe predicate frame caches (top to bottom) for reuse
- [x] Modified `get_pred()` to access condition via `frame.cond`
- [x] Fixed assertion message in `exit_context_of` to show leaked predicate condition
- [x] Verified with FSM tests and CI test driver — all passing
- [x] No linter errors

## 2. Changes made (areas)

### Backend/Builder
- `python/assassyn/builder/__init__.py`: Added `PredicateFrame` class with `cond` and `array_cache` fields. Simplified `push_predicate` to create and push a frame; `pop_predicate` to pop it. Removed `array_read_layers_by_cond` tracking. Updated module exit assertion to show leaked predicate.

### Frontend/IR
- `python/assassyn/ir/array.py`: Re-enabled `__getitem__` caching. Probes active predicate frame caches (top to bottom) for an existing read. If none found, creates a new `ArrayRead` and inserts it into the top predicate frame's cache.

### Intrinsics
- `python/assassyn/ir/expr/intrinsic.py::get_pred()`: Already supports frame-based access by iterating `frame.cond` instead of raw `cond` values.

## 3. Technical decisions and insights

**Design decision — pairing predicate with cache in a single data structure**: Instead of maintaining separate per-block cache layers, we embed the cache dictionary directly in each predicate frame. This ensures cache lifetime matches predicate lifetime (push/pop), simplifying invalidation and avoiding leaks across predicate boundaries. When a predicate is pushed, a new empty cache is created; when popped, the entire cache is discarded, ensuring reads created under that predicate are not reused afterward.

**Simplification over previous attempts**: Early iterations tried to maintain a separate `array_read_layers_by_cond` mapping and complex per-block layer management. The final design leverages the existing `cond_stack` infrastructure by storing both the condition and its cache together, eliminating cross-component coordination complexity.

**Nested predicate behavior**: With nested predicates, each level has its own cache. The lookup algorithm probes from top (most nested) to bottom, allowing outer-scope reads to be reused within inner predicates while still preventing inner-scope reads from leaking out after their predicate pops.

**No changes to block enter/exit behavior**: Block context management remains unchanged. The array-read cache is now fully managed by predicate push/pop operations, which naturally aligns with semantic lifetimes.

**User fix — assertion message**: User updated the module-exit assertion to print the leaked predicate condition, improving debugging for violated invariants.

## 4. Test status

- All existing CI tests pass, including `test_fsm.py` and `test_driver.py`
- Array read deduplication works correctly within predicate scopes
- No cache leaks observed across predicate boundaries
- No regressions in other modules

