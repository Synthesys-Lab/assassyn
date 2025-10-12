# TODO: Fix Array Module Documentation and Implementation Issues

## Goal

Fix inconsistencies and implementation issues found in the `ir/array.py` module during documentation review, and ensure the `attr` parameter functionality is properly implemented or removed.

## Action Items

### Document Development

- [x] **Update design document for array module**: The documentation has been reorganized according to new standards with proper Summary, Exposed Interfaces, and Internal Helpers sections. The missing `Slice` class has been documented.

### Coding Development

#### Issue 1: Inconsistent `attr` Parameter Handling

**Problem**: The `RegArray` function accepts an `attr` parameter but it's never actually used. The parameter is processed (defaulted to empty list if None) but never passed to the `Array` constructor or used anywhere else in the code.

**Current State**:
```python
def RegArray(scalar_ty: DType, size: int, initializer: list = None, name: str = None, attr: list = None):
    attr = attr if attr is not None else []  # Processed but never used
    res = Array(scalar_ty, size, initializer)  # attr not passed
    # ... rest of function
```

**Required Changes**:
1. **Option A - Remove unused parameter**: Remove the `attr` parameter from `RegArray` function signature and update all call sites
2. **Option B - Implement functionality**: Modify `Array` constructor to accept and use the `attr` parameter

**Recommendation**: Choose Option A (remove unused parameter) as there's no evidence of `attr` being used anywhere in the codebase, and it adds unnecessary complexity.

**Files to modify**:
- `python/assassyn/ir/array.py` - Remove `attr` parameter from `RegArray` function
- `python/assassyn/ir/array.md` - Update documentation to remove `attr` parameter references
- Search and update any call sites that pass `attr` parameter (if any exist)

**Commit message**: "Remove unused attr parameter from RegArray function"

#### Issue 2: Missing Array Constructor Documentation

**Problem**: The `Array` class constructor is not documented in the current documentation structure.

**Required Changes**:
1. Add `__init__` method documentation to the `Array` class section in `ir/array.md`
2. Document the constructor parameters and their purposes
3. Explain the initialization process and default values

**Files to modify**:
- `python/assassyn/ir/array.md` - Add `__init__` method documentation

**Commit message**: "Add Array constructor documentation"

### Testing

- [ ] **Run existing tests**: Ensure all existing tests pass after removing the `attr` parameter
- [ ] **Verify functionality**: Test that array creation, naming, and write port functionality still works correctly
- [ ] **Check for breaking changes**: Verify that no external code depends on the `attr` parameter

### Documentation Updates

- [x] **Update array.md**: Documentation has been reorganized according to new standards
- [ ] **Update any related documentation**: Check if other files reference the `attr` parameter and update accordingly

## Notes

- The `attr` parameter appears to be a legacy feature that was never fully implemented
- No test cases or usage examples were found that actually use the `attr` parameter
- The current implementation always initializes `attr` to an empty list in the `Array` constructor
- Removing the unused parameter will simplify the API and eliminate confusion

## Dependencies

- This change may affect any external code that passes the `attr` parameter to `RegArray`
- Need to verify that the naming manager and builder integration still works correctly
- Should coordinate with any ongoing work on array functionality