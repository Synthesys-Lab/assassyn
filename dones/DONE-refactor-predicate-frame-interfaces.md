# Refactor PredicateFrame and ModuleContext Interfaces

## 0. Goal achieved / current state

Added type-safe cache management interfaces to `PredicateFrame` and enhanced type annotations to both `PredicateFrame` and `ModuleContext`. The changes improve encapsulation by replacing direct dictionary access with proper methods, making the cache protocol explicit and type-safe. String annotations are used for type hints to avoid circular import issues.

## 1. Checklist of action items completed

- [x] Keep Module and Value imports in TYPE_CHECKING block to avoid circular imports
- [x] Add type annotations to PredicateFrame: array_cache as dict[tuple['Array', 'Value'], 'ArrayRead']
- [x] Add cache management methods to PredicateFrame: get_cached_read(), cache_read(), has_cached_read()
- [x] Add proper type annotation to ModuleContext.cond_stack as list[PredicateFrame]
- [x] Refactor array.py to use new PredicateFrame cache methods
- [x] Update __init__.md to document new interfaces
- [x] Update array.md to reflect predicate-aware caching
- [x] Fix assertion in exit_context_of to check cond_stack before accessing
- [x] Run test suite and linter - all passing
- [x] Stage and commit with proper message following pre-commit guidelines

## 2. Changes made (areas)

### Backend/Builder
- `python/assassyn/builder/__init__.py`: 
  - Added `get_cached_read()`, `cache_read()`, and `has_cached_read()` methods to `PredicateFrame` class
  - Added type annotation `array_cache: dict[tuple['Array', 'Value'], 'ArrayRead']` to PredicateFrame
  - Added type annotation `cond_stack: list[PredicateFrame]` to ModuleContext
  - Fixed assertion in `exit_context_of` to check if cond_stack is non-empty before accessing for error message
  - Added ArrayRead to TYPE_CHECKING imports

### Frontend/IR
- `python/assassyn/ir/array.py`: 
  - Refactored `__getitem__` to use `frame.get_cached_read(self, index)` instead of direct dictionary access
  - Refactored cache insertion to use `pred_stack[-1].cache_read(self, index, res)` instead of direct assignment

### Documentation
- `python/assassyn/builder/__init__.md`: 
  - Added comprehensive documentation for `PredicateFrame` class including all cache management methods
  - Updated `ModuleContext` documentation to reflect `cond_stack: list[PredicateFrame]`
  - Updated "Naming and Caches" section to document predicate-frame-scoped caching
  
- `python/assassyn/ir/array.md`: 
  - Updated `__getitem__` explanation to reflect predicate-aware caching mechanism
  - Added "Cache Protocol" section documenting cache probing, insertion, invalidation, and scope

### Other
- `python/assassyn/ir/block.py`: 
  - Removed unused `ir_builder` import
  - Added pylint disable for line-too-long on docstring

## 3. Technical decisions and insights

**Type annotations using string literals**: After attempting to move `Module` and `Value` imports out of TYPE_CHECKING block, we encountered circular import issues. The solution was to keep these imports in TYPE_CHECKING and use string annotations ('Value', 'Module', 'ArrayRead') for type hints in class attributes and method signatures. This provides type checking benefits without runtime import costs.

**Cache management encapsulation**: Adding explicit methods (`get_cached_read`, `cache_read`, `has_cached_read`) instead of direct dictionary access improves:
- Type safety: Method signatures enforce correct parameter types
- Encapsulation: Cache implementation details are hidden
- Maintainability: Cache protocol is explicit and easy to modify
- Readability: Method names clearly convey intent

**Predicate frame cache lifetime**: The cache is scoped to each predicate frame, ensuring proper invalidation when predicates are popped. This prevents array reads created under a predicate from being reused after the predicate expires, which is essential for FSM and other conditional execution patterns.

**Assertion fix**: The original assertion accessed `ctx.cond_stack[-1].cond` without checking if the stack was empty first, causing IndexError when the assertion fired. The fix checks `if ctx.cond_stack:` before accessing the top element, ensuring the error message is only constructed when there's actually a leaked predicate to report.

## 4. Test status

- All existing CI tests pass, including test_driver.py
- All unit tests pass (51 tests in ~62 seconds)
- All linter checks pass (pylint rating 10.00/10)
- Array read deduplication works correctly within predicate scopes
- No cache leaks observed across predicate boundaries
- No regressions in other modules
