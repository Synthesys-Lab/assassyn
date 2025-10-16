# @enforce_type Decorator

## Section 0. Summary

The `@enforce_type` decorator provides runtime type validation for function arguments based on their type annotations. This module implements a type enforcement system that validates function parameters against their declared types at runtime, raising `TypeError` exceptions when type mismatches occur. The system supports simple types, Optional types, Union types, generic types, and Any types, with zero performance overhead when types are correct.

## Section 1. Exposed Interfaces

### `@enforce_type`

```python
@enforce_type
def function_name(param1: Type1, param2: Type2) -> ReturnType:
    """Function with runtime type validation."""
    pass
```

**Purpose**: Decorator that wraps functions to provide runtime type validation of arguments.

**Parameters**: None (decorator)

**Returns**: Decorated function with type validation

**Raises**: `TypeError` if any argument doesn't match its type annotation

**Supported Types**:
- Simple types: `int`, `str`, `bool`, `float`, custom classes
- Optional types: `Optional[T]` or `Union[T, None]`
- Union types: `Union[A, B]`
- Generic types: `List[T]`, `Dict[K, V]`, `Tuple[...]` (structure validation only)
- Any type: `Any` (skips validation)

**Performance**: Zero overhead when types are correct, minimal overhead on validation failure.

### `validate_arguments(func, args, kwargs)`

```python
def validate_arguments(func: Callable[..., Any], args: tuple, kwargs: dict) -> Dict[str, Any]:
    """Validate arguments passed to a function against its type annotations."""
```

**Purpose**: Core validation logic that extracts function arguments and validates them against type annotations.

**Parameters**:
- `func` (Callable): Function to validate arguments for
- `args` (tuple): Positional arguments
- `kwargs` (dict): Keyword arguments

**Returns**: `Dict[str, Any]` - Dictionary of validated arguments

**Raises**: `TypeError` if any argument doesn't match its type annotation

### `check_type(value, expected_type)`

```python
def check_type(value: Any, expected_type: Any) -> bool:
    """Check if a value matches the expected type annotation."""
```

**Purpose**: Atomic type checking primitive that validates a single value against a type annotation.

**Parameters**:
- `value` (Any): Value to check
- `expected_type` (Any): Type annotation to check against

**Returns**: `bool` - True if value matches type

**Raises**: `TypeError` if value doesn't match expected type

## Section 2. Internal Helpers

### `_check_simple_type(value, expected_type)`

```python
def _check_simple_type(value: Any, expected_type: type) -> bool:
    """Check simple type (non-generic)."""
```

**Purpose**: Validates simple types (int, str, bool, float, custom classes) with special handling for built-in types to avoid bool/int confusion.

**Parameters**:
- `value` (Any): Value to validate
- `expected_type` (type): Expected type

**Returns**: `bool` - True if validation passes

**Raises**: `TypeError` if validation fails

**Technical Details**: Uses exact type matching (`type(value) is expected_type`) for built-in types to prevent `isinstance(True, int)` from returning True.

### `_check_union_type(value, expected_type)`

```python
def _check_union_type(value: Any, expected_type: Any) -> bool:
    """Check Union type (including Optional)."""
```

**Purpose**: Validates Union types including Optional types, handling the special case of `Optional[T]` (Union[T, None]).

**Parameters**:
- `value` (Any): Value to validate
- `expected_type` (Any): Union type annotation

**Returns**: `bool` - True if validation passes

**Raises**: `TypeError` if validation fails

**Technical Details**: Special handling for Optional types where None is accepted, otherwise validates against the non-None variant.

### `_check_generic_type(value, origin)`

```python
def _check_generic_type(value: Any, origin: Any) -> bool:
    """Check generic type structure."""
```

**Purpose**: Validates generic type structure (List, Dict, Tuple) without validating contents for performance reasons.

**Parameters**:
- `value` (Any): Value to validate
- `origin` (Any): Generic type origin (list, dict, tuple)

**Returns**: `bool` - True if validation passes

**Raises**: `TypeError` if validation fails

**Technical Details**: Only validates that the value is the correct container type (list vs dict vs tuple), not the contents.

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

## See Also

- [Design Documentation](../../docs/design/internal/enforce_type.md) - High-level design and implementation details
