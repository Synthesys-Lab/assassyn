# TODO: Documentation Review - Visitor, Arithmetic, Array, Call, and Comm Modules

**Date**: 2024-12-19  
**Scope**: Documentation review and reorganization of 5 IR expression modules  
**Status**: Completed documentation review, issues identified below

---

## Summary

Reviewed and reorganized documentation for 5 modules according to new documentation standards:
- `ir/visitor.py` → `ir/visitor.md`
- `ir/expr/arith.py` → `ir/expr/arith.md` 
- `ir/expr/array.py` → `ir/expr/array.md`
- `ir/expr/call.py` → `ir/expr/call.md`
- `ir/expr/comm.py` → `ir/expr/comm.md`

All modules have been moved to the DONE section in `DOCUMENTATION-STATUS.md`.

---

## Issues Requiring Human Intervention

### 1. BinaryOp.dtype Property - Addition Carry Handling

**File**: `python/assassyn/ir/expr/arith.py` (lines 82-84)  
**Issue**: The TODO comment indicates addition should be `bits + 1` for carry, but current implementation uses `max(bits)`

```python
if self.opcode in [BinaryOp.ADD]:
    # TODO(@were): Make this bits + 1
    bits = max(self.lhs.dtype.bits, self.rhs.dtype.bits)
```

**Decision Needed**: 
- Should addition operations automatically add 1 bit for carry?
- What are the implications for existing code that relies on current behavior?
- Is this a breaking change or can it be implemented safely?

### 2. ArrayWrite Module Context Dependency

**File**: `python/assassyn/ir/expr/array.py` (lines 26-30)  
**Issue**: ArrayWrite relies on builder singleton for module context when not provided

```python
if module is None:
    # pylint: disable=import-outside-toplevel
    from ...builder import Singleton
    module = Singleton.builder.current_module
```

**Decision Needed**:
- Is this singleton dependency acceptable for the architecture?
- Should there be explicit module passing instead of implicit singleton access?
- Are there edge cases where `current_module` might be None?

### 3. Bind Operation Validation

**File**: `python/assassyn/ir/expr/call.py` (lines 99-101)  
**Issue**: The `set_fifo_depth` method has a commented-out `break` statement

```python
if push.fifo.name == name:
    push.fifo_depth = depth
    matches = matches + 1
    #break
```

**Decision Needed**:
- Should the loop break after finding the first match?
- Is the current behavior (allowing multiple matches) intentional?
- What happens if multiple pushes have the same FIFO name?

### 4. Comm Module Error Handling

**File**: `python/assassyn/ir/expr/comm.py` (lines 34-36)  
**Issue**: `concat` function requires at least 2 arguments but other functions don't validate argument count

```python
def concat(*args):
    if len(args) < 2:
        raise ValueError("concat requires at least two arguments")
```

**Decision Needed**:
- Should other commutative functions also validate minimum argument count?
- What should happen with single-argument calls to `add`, `mul`, etc.?
- Is the current behavior consistent across all functions?

---

## Potential Code Improvements

### 1. Visitor Pattern Type Safety

**File**: `python/assassyn/ir/visitor.py`  
**Suggestion**: Consider adding type hints for better IDE support and static analysis

```python
def dispatch(self, node) -> None:
    # Could be more specific about node types
```

### 2. Arithmetic Operation Constants

**File**: `python/assassyn/ir/expr/arith.py`  
**Suggestion**: Consider using enum instead of integer constants for better maintainability

```python
# Current: ADD = 200
# Suggested: class BinaryOpType(Enum): ADD = 200
```

### 3. Array Operation Error Messages

**File**: `python/assassyn/ir/expr/array.py`  
**Suggestion**: More descriptive error messages for debugging

```python
# Current: assert isinstance(arr, Array), f'{type(arr)} is not an Array!'
# Could include expected vs actual type information
```

---

## Documentation Quality Assessment

### Strengths
- All modules now follow consistent documentation structure
- Cross-references to design documents provide context
- Method explanations include project-specific knowledge
- Function signatures are properly documented

### Areas for Future Improvement
- Consider adding usage examples for complex operations
- Add performance considerations for large-scale operations
- Include migration guides for any breaking changes

---

## Next Steps

1. **Review identified issues** with the development team
2. **Make decisions** on the architectural questions above
3. **Implement fixes** for any inconsistencies
4. **Update tests** if behavior changes are made
5. **Consider** the suggested code improvements for future iterations

---

## Files Modified

- `python/assassyn/ir/visitor.md` - Reorganized and enhanced
- `python/assassyn/ir/expr/arith.md` - Reorganized and enhanced  
- `python/assassyn/ir/expr/array.md` - Reorganized and enhanced
- `python/assassyn/ir/expr/call.md` - Reorganized and enhanced
- `python/assassyn/ir/expr/comm.md` - Reorganized and enhanced
- `todos/DOCUMENTATION-STATUS.md` - Updated progress tracking
