# TODO: Fix Array Module Documentation and Implementation Issues

## Goal

Fix inconsistencies and unclear behaviors identified in the `ir/array.py` module during documentation review.

## Action Items

### Document Development

- **Issue Analysis**: During the documentation review of `ir/array.py`, several inconsistencies and unclear behaviors were identified that need to be addressed.

### Coding Development

#### 1. Fix Slice Class Property Inconsistencies

**Problem**: The `Slice` class has inconsistent property documentation and type annotations:

- `l` and `r` properties have incorrect docstrings ("Get the value to slice" instead of "Get the left/right bound")
- Type annotations claim these properties return `int`, but they actually return `UInt` values stored in `_operands`
- The `dtype` property assumes `l` and `r` are `Const` values, but they're actually `UInt` values

**Root Cause**: The constructor converts `int` literals to `UInt` values using `to_uint()`, but the property type annotations and docstrings weren't updated to reflect this.

**Required Changes**:
1. Update `l` and `r` property docstrings to correctly describe their purpose
2. Fix type annotations to reflect that they return `UInt` values, not `int`
3. Update the `dtype` property implementation to handle `UInt` values correctly
4. Consider whether the properties should return the raw `UInt` values or extract the integer values

**Files to Modify**:
- `python/assassyn/ir/array.py`: Fix property docstrings, type annotations, and `dtype` implementation
- `python/assassyn/ir/array.md`: Update documentation to reflect the corrected behavior

#### 2. Clarify Array Write Port Semantics

**Problem**: The relationship between `Array.__setitem__` and the `WritePort` mechanism could be clearer in the implementation.

**Current Behavior**: `__setitem__` uses `self & current_module` to get a write port, but this creates a dependency on the current module context that may not be obvious to users.

**Required Changes**:
1. Add more explicit error handling when no current module is available
2. Consider adding a comment explaining the implicit write port creation
3. Document the relationship between direct assignment (`array[index] = value`) and explicit write port usage (`(array & module)[index] <= value`)

**Files to Modify**:
- `python/assassyn/ir/array.py`: Add error handling and comments
- `python/assassyn/ir/array.md`: Already documented, but could be enhanced

#### 3. Verify Array Read Caching Behavior

**Problem**: The array read caching mechanism in `__getitem__` uses block-scoped caching, but the exact semantics of when cache entries are invalidated could be clearer.

**Current Behavior**: Cache is keyed by `(array, index)` within the current block, but it's unclear when the cache is cleared or if it persists across different execution contexts.

**Required Investigation**:
1. Verify that the caching behavior is correct for all use cases
2. Document when cache entries are invalidated
3. Consider whether the caching strategy is optimal for all scenarios

**Files to Investigate**:
- `python/assassyn/ir/array.py`: Review caching implementation
- `python/assassyn/builder/`: Check how `array_read_cache` is managed

### Testing Requirements

- **Unit Tests**: Add tests for the corrected `Slice` class properties
- **Integration Tests**: Verify that array read caching works correctly across different scenarios
- **Regression Tests**: Ensure that existing array functionality continues to work after fixes

### Documentation Updates

- **Update array.md**: Reflect any changes made to the implementation
- **Update related documentation**: Ensure consistency across all array-related documentation

## Notes

- The `Slice` class issues are the most critical as they involve incorrect type annotations and documentation
- The array write port semantics are already well-documented but could benefit from clearer error handling
- The caching behavior investigation is lower priority but should be addressed for completeness

## Dependencies

- Requires understanding of the `UInt` and `Const` type system
- Requires knowledge of the builder's block management system
- May require coordination with other modules that use `Slice` operations