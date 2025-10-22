<!-- ed9e81c8-4f11-4aa8-b2d2-12433b7a0b13 2da56c25-1e60-477c-8de0-2b16a8b38ad8 -->
# Hoist name Attribute to Value Base Class

## Overview

Add a universal `name` attribute to the `Value` base class to provide consistent naming across all IR nodes. This will:

1. Eliminate the need for `_safe_getattr` in the type-oriented namer
2. Provide a cleaner, more uniform interface for accessing names
3. Simplify the naming logic throughout the codebase

## Current State Analysis

### Classes with `name` attributes:

- **`Module`**: Has `name: str` (explicit attribute, set in `__init__`)
- **`Port`**: Has `name: str` (set to `None` in `__init__`, later assigned)
- **`Array`**: Has `name` property with getter/setter (uses `_name` internally)
- **`Downstream`**: Has `name` property with getter/setter (uses `_name` internally)
- **`Wire`**: Has `name` set to `None` in `__init__`

### Classes without `name` attributes:

- **`Value`**: Base class with no attributes
- **`Expr`**: Subclass of `Value`, no `name` attribute
- All expression classes: `BinaryOp`, `UnaryOp`, `ArrayRead`, `ArrayWrite`, `FIFOPop`, `FIFOPush`, `Cast`, `Slice`, `Concat`, `Select`, etc.
- **`Const`**: Has `dtype` and `value`, no `name`

### Key Usage in `type_oriented_namer.py`:

- Lines 83-89: `_entity_name()` checks `__assassyn_semantic_name__` then `name`
- Lines 195-197: `get_prefix_for_type()` falls back to `name` attribute
- Uses `_safe_getattr()` to avoid triggering `__getattr__` methods on expressions

## Implementation Plan

### 1. Add `name` attribute to `Value` base class

**File: `python/assassyn/ir/value.py`**

Add after line 10 (class definition):

```python
class Value:
    '''Base class for overloading arithmetic operations in the frontend'''
    
    name: str | None  # Optional name for this value
```

### 2. Initialize `name` in `Expr.__init__()`

**File: `python/assassyn/ir/expr/expr.py`**

In `Expr.__init__()` around line 53-57, add:

```python
def __init__(self, opcode, operands: list):
    '''Initialize the expression with an opcode'''
    self.opcode = opcode
    self.loc = self.parent = None
    self.name = None  # Initialize name attribute
    self._operands = []
    # ... rest of init
```

### 3. Initialize `name` in `Const.__init__()`

**File: `python/assassyn/ir/const.py`**

In `Const.__init__()` around line 14-17, add:

```python
def __init__(self, dtype: DType, value: int):
    assert dtype.inrange(value), f"Value {value} is out of range for {dtype}"
    self.dtype = dtype
    self.value = value
    self.name = None  # Initialize name attribute
```

### 4. Update classes that already have `name` to ensure compatibility

**`Module`** (already has `name: str`) - no changes needed, name is assigned in `__init__`

**`Port`** (already has `name: str`) - already initializes to `None` in `__init__`, no changes needed

**`Array`** - Keep the property-based approach since it has special logic for `__assassyn_semantic_name__`

**`Downstream`** - Keep the property-based approach since it has special logic for `__assassyn_semantic_name__`

**`Wire`** - Already initializes `name = None` in `__init__`, no changes needed

### 5. Simplify `type_oriented_namer.py`

**File: `python/assassyn/builder/type_oriented_namer.py`**

- **Remove `_safe_getattr` method** (lines 68-74)
- **Simplify `_entity_name` method** (lines 76-91):
  ```python
  def _entity_name(self, entity: Any) -> Optional[str]:
      """Extract a meaningful name from an entity."""
      if entity is None:
          return None
      
      entity = unwrap_operand(entity)
      
      # Check semantic name first
      semantic = getattr(entity, '__assassyn_semantic_name__', None)
      if isinstance(semantic, str) and semantic:
          return self._sanitize(semantic)
      
      # Fall back to name attribute
      name_attr = getattr(entity, 'name', None)
      if isinstance(name_attr, str) and name_attr:
          return self._sanitize(name_attr)
      
      return None
  ```

- **Simplify `get_prefix_for_type` method** (lines 195-197):
  ```python
  # Fallback to name attribute or 'val'
  name_attr = getattr(node, 'name', None)
  if isinstance(name_attr, str):
      return self._sanitize(name_attr)
  ```


### 6. Verify no regressions

Run tests to ensure:

- All existing name assignments still work
- No new AttributeErrors are raised
- Naming logic produces the same results

## Files to Modify

1. `python/assassyn/ir/value.py` - Add `name` attribute declaration
2. `python/assassyn/ir/expr/expr.py` - Initialize `name = None` in `Expr.__init__()`
3. `python/assassyn/ir/const.py` - Initialize `name = None` in `Const.__init__()`
4. `python/assassyn/builder/type_oriented_namer.py` - Remove `_safe_getattr`, simplify name access

## Benefits

1. **Simpler code**: No need for `_safe_getattr` defensive programming
2. **Uniform interface**: All `Value` subclasses have a `name` attribute
3. **Better type safety**: Static analyzers can see `name` is always present
4. **Easier debugging**: Can always check `.name` without AttributeError
5. **Follows Python conventions**: Explicit attributes are better than hasattr checks

### To-dos

- [ ] Add name attribute declaration to Value base class
- [ ] Initialize name = None in Expr.__init__()
- [ ] Initialize name = None in Const.__init__()
- [ ] Remove _safe_getattr and simplify name access in type_oriented_namer.py
- [ ] Run tests to verify no regressions