# Type Enforcement Decorator

## Overview

The `@enforce_type` decorator provides runtime type validation for function arguments based on their type annotations. This addresses Phase 2 documentation plan point #3 "Type System and Error Handling Documentation" by providing fundamental runtime enforcement of type contracts throughout the codebase.

## Motivation

During Phase 1 documentation, several type annotation inconsistencies were identified:

- `Slice` class type annotations claim `int` return but actually return `UInt` values
- `BinaryOp.dtype` property has TODO for carry bit handling  
- `Record.attributize` has incomplete implementation with TODO comments
- `Const` class has 32-bit limitation that's not well documented
- Several methods lack runtime validation despite documented restrictions

The `@enforce_type` decorator provides a systematic solution by extracting and generalizing the validation logic already present in `factory.py`.

## API Design

### Basic Usage

```python
from assassyn.utils import enforce_type

@enforce_type
def example_function(value: int, name: str, optional: Optional[Value] = None) -> str:
    """Function with type enforcement."""
    return f"{name}: {value}"
```

### Advanced Usage

```python
from typing import List, Dict, Union, Optional, Any
from assassyn.utils import enforce_type

@enforce_type
def complex_function(
    items: List[int],
    mapping: Dict[str, Value],
    variant: Union[int, str],
    nullable: Optional[Module] = None,
    anything: Any = None
) -> None:
    """Function with complex type annotations."""
    pass
```

## Type Validation Rules

### Supported Types

1. **Simple Types**: `int`, `str`, `bool`, `float`, custom classes
2. **Optional Types**: `Optional[T]` or `Union[T, None]` - accepts `T` or `None`
3. **Union Types**: `Union[A, B]` - accepts any of the specified types
4. **Generic Types**: `List[T]`, `Dict[K, V]`, `Tuple[...]` - validates structure only
5. **Any Type**: `Any` - skips validation (trust the caller)

### Validation Behavior

- **Valid types**: Pass through unchanged, no performance overhead
- **Invalid types**: Raise `TypeError` with clear message:
  ```
  TypeError: Argument 'value' must be of type int, got str
  ```
- **Complex annotations**: Graceful fallback - trust the caller for unsupported patterns
- **Missing annotations**: Skip validation (no annotation = no enforcement)

### Error Handling

The decorator provides detailed error messages including:
- Parameter name that failed validation
- Expected type from annotation
- Actual type of the provided value
- Function name for context

## Integration with Existing Code

### Factory Decorator Integration

The decorator extracts validation logic from `factory.py`:

```python
# Before: factory-specific validation
def _validate_outer_arguments(func, args, kwargs):
    # 50+ lines of validation logic...

# After: shared validation
from assassyn.utils.enforce_type import validate_arguments

def _validate_outer_arguments(func, args, kwargs):
    return validate_arguments(func, args, kwargs)
```

This maintains backward compatibility while enabling reuse across the codebase.

## Implementation Details

### Core Functions

- `@enforce_type` - Main decorator that wraps functions
- `validate_arguments(func, args, kwargs)` - Core validation logic
- `check_type(value, expected_type)` - Type checking helper

### Type Checking Algorithm

1. Extract type annotations using `get_type_hints()`
2. Bind arguments to parameter names using `inspect.signature()`
3. For each argument:
   - Skip if no annotation
   - Handle `Any` type (skip validation)
   - Handle `Union` types (check against all variants)
   - Handle `Optional` types (check against non-None variant)
   - Handle simple types (direct `isinstance()` check)
   - Handle generics (structure validation only)
4. Raise `TypeError` on first validation failure

### Performance Characteristics

- **Zero overhead** when types are correct (only annotation extraction)
- **Minimal overhead** on validation failure (early exit)
- **No caching** - annotations extracted on each call (simple and reliable)

## Usage Examples

### Expression Module Integration

```python
# ir/expr/array.py
@enforce_type
def __init__(self, arr: Array, idx: Value, val: Value, module: ModuleBase = None):
    # Runtime validation ensures correct types
    pass
```

### Const Module Integration

```python
# ir/const.py
@enforce_type
def __getitem__(self, x: slice) -> UInt:
    # Runtime validation + 32-bit check
    assert 0 < bits <= 32, "TODO: Support more than 32 bits later"
    pass
```

### Module Base Integration

```python
# ir/module/base.py
@enforce_type
def triggered(self) -> PureIntrinsic:
    # Runtime validation + downstream-only check
    if not self._is_downstream_module():
        raise RuntimeError("triggered() only usable in downstream modules")
    pass
```

## Future Improvements

1. **Nested Generic Validation**: Validate `List[List[int]]` contents
2. **Protocol Support**: Validate against `typing.Protocol` classes
3. **Performance Optimization**: Cache annotation extraction
4. **Custom Validators**: Allow custom validation functions
5. **Warning Mode**: Optional warning instead of error for development

## Design Decisions

1. **Structure-only generics**: Validates `List` vs `Dict` but not contents (performance vs correctness tradeoff)
2. **No caching**: Simpler implementation, annotations rarely change
3. **Early exit**: Fail fast on first error (better debugging experience)
4. **Graceful fallback**: Complex annotations don't break the decorator
5. **Preserve metadata**: Uses `functools.wraps` to maintain function signatures

## Technical Insights

- **Annotation extraction**: Uses `get_type_hints()` for forward reference support
- **Union handling**: Special case for `Optional` (common pattern)
- **Error messages**: Include context for easier debugging
- **Backward compatibility**: Existing code continues to work unchanged
- **Extensibility**: Easy to add new type patterns as needed
