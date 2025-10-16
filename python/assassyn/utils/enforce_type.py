"""Type enforcement decorator for runtime type validation.

This module provides a general-purpose @enforce_type decorator that validates
function arguments against their type annotations at runtime. It extracts and
generalizes the validation logic from factory.py for reuse across the codebase.
"""

from __future__ import annotations

import functools
import inspect
from typing import (
    Any, Callable, Dict, List, Union,
    get_args, get_origin, get_type_hints
)


def _check_simple_type(value: Any, expected_type: type) -> bool:
    """Check simple type (non-generic)."""
    # Use exact type matching for built-in types to avoid bool/int confusion
    if expected_type in (int, str, float, bool):
        # For built-in types, we need exact type matching to avoid bool/int confusion
        # pylint: disable=unidiomatic-typecheck
        if type(value) is not expected_type:
            raise TypeError(f"Expected {expected_type.__name__}, got {type(value).__name__}")
    else:
        # For custom types, use isinstance
        if not isinstance(value, expected_type):
            raise TypeError(f"Expected {expected_type.__name__}, got {type(value).__name__}")
    return True


def _check_union_type(value: Any, expected_type: Any) -> bool:
    """Check Union type (including Optional)."""
    args_hint = get_args(expected_type)

    # Special case for Optional (Union[T, None])
    non_none = [arg for arg in args_hint if arg is not type(None)]
    if len(non_none) == 1 and len(non_none) != len(args_hint):
        # This is Optional[T] - handle None case
        if value is None:
            return True
        # Check against the non-None type
        try:
            return check_type(value, non_none[0])
        except TypeError as exc:
            raise TypeError(f"Expected {non_none[0].__name__}, got {type(value).__name__}") from exc

    # Regular Union - check against all variants
    for variant in args_hint:
        try:
            if check_type(value, variant):
                return True
        except TypeError:
            continue

    # None of the variants matched
    variant_names = [getattr(arg, '__name__', str(arg)) for arg in args_hint]
    raise TypeError(f"Expected {' or '.join(variant_names)}, got {type(value).__name__}")


def _check_generic_type(value: Any, origin: Any) -> bool:
    """Check generic type structure."""
    if origin in (list, List):
        if not isinstance(value, list):
            raise TypeError(f"Expected list, got {type(value).__name__}")
        return True

    if origin in (dict, Dict):
        if not isinstance(value, dict):
            raise TypeError(f"Expected dict, got {type(value).__name__}")
        return True

    if origin in (tuple, tuple):
        if not isinstance(value, tuple):
            raise TypeError(f"Expected tuple, got {type(value).__name__}")
        return True

    # For unsupported generics, trust the caller
    return True


def check_type(value: Any, expected_type: Any) -> bool:
    """Check if a value matches the expected type annotation.

    Args:
        value: The value to check
        expected_type: The type annotation to check against

    Returns:
        True if the value matches the type

    Raises:
        TypeError: If the value doesn't match the expected type
    """
    # Handle Any type - skip validation
    if expected_type is Any:
        return True

    # Handle simple types
    if isinstance(expected_type, type):
        return _check_simple_type(value, expected_type)

    # Handle Union types (including Optional)
    origin = get_origin(expected_type)
    if origin is Union:
        return _check_union_type(value, expected_type)

    # Handle generic types (List, Dict, etc.) - structure validation only
    if origin is not None:
        return _check_generic_type(value, origin)

    # For unsupported complex annotations, trust the caller
    return True


def validate_arguments(func: Callable[..., Any], args: tuple, kwargs: dict) -> Dict[str, Any]:
    """Validate arguments passed to a function against its type annotations.

    Args:
        func: The function to validate arguments for
        args: Positional arguments
        kwargs: Keyword arguments

    Returns:
        Dictionary of validated arguments

    Raises:
        TypeError: If any argument doesn't match its type annotation
    """
    signature = inspect.signature(func)
    bound_arguments = signature.bind(*args, **kwargs)
    bound_arguments.apply_defaults()

    try:
        annotations = get_type_hints(func)
    except NameError:
        # Forward references under TYPE_CHECKING can't be resolved at runtime
        # Fall back to raw annotations (strings won't be validated)
        annotations = getattr(func, '__annotations__', {})

    validated: Dict[str, Any] = {}
    for name, value in bound_arguments.arguments.items():
        expected = annotations.get(name)
        if expected is None:
            # No annotation - skip validation
            validated[name] = value
            continue

        # Skip validation for unresolved forward references (strings)
        if isinstance(expected, str):
            validated[name] = value
            continue

        # Validate the type
        try:
            check_type(value, expected)
        except TypeError as exc:
            # Re-raise with argument name context
            # Handle Union types specially for better error messages
            if get_origin(expected) is Union:
                args_hint = get_args(expected)
                # Special case for Optional (Union[T, None])
                non_none = [arg for arg in args_hint if arg is not type(None)]
                if len(non_none) == 1 and len(non_none) != len(args_hint):
                    # This is Optional[T]
                    type_str = non_none[0].__name__
                else:
                    # Regular Union
                    variant_names = [getattr(arg, '__name__', str(arg)) for arg in args_hint]
                    type_str = ' or '.join(variant_names)
            elif get_origin(expected) in (list, List):
                type_str = 'list'
            elif get_origin(expected) in (dict, Dict):
                type_str = 'dict'
            elif get_origin(expected) in (tuple, tuple):
                type_str = 'tuple'
            else:
                type_str = getattr(expected, '__name__', str(expected))
            raise TypeError(
                f"Argument '{name}' must be of type {type_str}, got {type(value).__name__}"
            ) from exc
        validated[name] = value

    return validated


def enforce_type(func: Callable[..., Any]) -> Callable[..., Any]:
    """Decorator that enforces type annotations at runtime.

    This decorator validates function arguments against their type annotations
    before calling the function. It supports:
    - Simple types (int, str, bool, etc.)
    - Optional types (Optional[T] or Union[T, None])
    - Union types (Union[A, B])
    - Generic types (List[T], Dict[K, V]) - structure validation only
    - Any type (skips validation)

    Args:
        func: The function to decorate

    Returns:
        Decorated function with type validation

    Example:
        @enforce_type
        def example(value: int, name: str, optional: Optional[Value] = None) -> str:
            return f"{name}: {value}"
    """
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        validated_args = validate_arguments(func, args, kwargs)
        return func(**validated_args)

    return wrapper


__all__ = ['enforce_type', 'validate_arguments', 'check_type']
