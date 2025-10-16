# @enforce_type Decorator

## Overview

The `@enforce_type` decorator provides runtime type validation for function arguments based on their type annotations.

## Interface

### `@enforce_type`

```python
from assassyn.type_utils import enforce_type

@enforce_type
def function_name(param1: Type1, param2: Type2) -> ReturnType:
    """Function with runtime type validation."""
    pass
```

**Parameters:** None (decorator)

**Returns:** Decorated function with type validation

**Raises:** `TypeError` if any argument doesn't match its type annotation

### `validate_arguments(func, args, kwargs)`

```python
from assassyn.type_utils import validate_arguments

validated = validate_arguments(func, args, kwargs)
```

**Parameters:**
- `func` (Callable): Function to validate arguments for
- `args` (tuple): Positional arguments
- `kwargs` (dict): Keyword arguments

**Returns:** `Dict[str, Any]` - Dictionary of validated arguments

**Raises:** `TypeError` if any argument doesn't match its type annotation

### `check_type(value, expected_type)`

```python
from assassyn.type_utils import check_type

is_valid = check_type(value, expected_type)
```

**Parameters:**
- `value` (Any): Value to check
- `expected_type` (Any): Type annotation to check against

**Returns:** `bool` - True if value matches type

**Raises:** `TypeError` if value doesn't match expected type

## Supported Types

- **Simple types**: `int`, `str`, `bool`, `float`, custom classes
- **Optional types**: `Optional[T]` or `Union[T, None]`
- **Union types**: `Union[A, B]`
- **Generic types**: `List[T]`, `Dict[K, V]`, `Tuple[...]` (structure validation only)
- **Any type**: `Any` (skips validation)

## Usage Examples

### Basic Function

```python
@enforce_type
def add_numbers(a: int, b: int) -> int:
    return a + b

# Valid usage
result = add_numbers(5, 3)  # Returns 8

# Invalid usage
add_numbers("5", 3)  # TypeError: Argument 'a' must be of type int, got str
```

### Optional Parameters

```python
@enforce_type
def process_value(value: int, name: Optional[str] = None) -> str:
    if name is None:
        return str(value)
    return f"{name}: {value}"

# Valid usage
process_value(42)  # Returns "42"
process_value(42, "answer")  # Returns "answer: 42"
```

### Union Types

```python
@enforce_type
def handle_input(data: Union[int, str]) -> str:
    return str(data)

# Valid usage
handle_input(123)  # Returns "123"
handle_input("hello")  # Returns "hello"
```

### Generic Types

```python
@enforce_type
def process_list(items: List[int]) -> int:
    return sum(items)

# Valid usage
process_list([1, 2, 3])  # Returns 6

# Invalid usage
process_list(["a", "b"])  # TypeError: Argument 'items' must be of type list, got list
```

## Error Messages

The decorator provides clear error messages:

```
TypeError: Argument 'param_name' must be of type ExpectedType, got ActualType
```

For Union types:
```
TypeError: Argument 'param_name' must be of type int or str, got bool
```

For Optional types:
```
TypeError: Argument 'param_name' must be of type int, got str
```

## Performance

- **Zero overhead** when types are correct
- **Minimal overhead** on validation failure
- **No caching** - annotations extracted on each call

## See Also

- [Design Documentation](../../../docs/design/internal/enforce_type.md) - High-level design and implementation details
