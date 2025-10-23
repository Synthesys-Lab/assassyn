"""Unified factory decorator for experimental frontend modules.

See factory.md for the design overview. This module exposes a generic
`@factory(<type>)` decorator together with a lightweight `Factory`
wrapper that normalises module construction across different module
flavours (e.g. standard modules, downstream logic, callbacks).
"""

from __future__ import annotations

import functools
from typing import (
    Any, Callable, Dict, Generic, Optional, Type, TypeVar
)

from assassyn.builder import Singleton
from assassyn.ir.block import Block
from assassyn.ir.value import Value
from assassyn.utils.enforce_type import validate_arguments

ModuleLike = TypeVar('ModuleLike')


class Factory(Generic[ModuleLike]):  # pylint: disable=too-few-public-methods
    """Generic wrapper returned by `@factory` decorated functions.

    Attributes:
        module: Underlying module instance produced by the decorator.
        pins: Optional set of combinational pins exposed via `expose`.
    """

    _specialisations: Dict[Type[Any], Type['Factory']] = {}

    def __init__(self, module: ModuleLike):
        self.module = module
        # Copy pins from module if they were set during construction
        self.pins: Optional[list[Value]] = getattr(module, 'pins', None)

    def expose(self, *pins: Value) -> 'Factory[ModuleLike]':
        """Expose combinational pins to upstream modules."""
        self.pins = list(pins)
        return self

    def __class_getitem__(cls, item: Type[Any]) -> Type['Factory']:
        """Return (and cache) a specialised Factory subclass for `item`."""
        if not isinstance(item, type):
            raise TypeError(f"Factory[...] expects a type, got {item!r}")
        if item not in cls._specialisations:
            subclass_name = f"{item.__name__}Factory"
            cls._specialisations[item] = type(subclass_name, (cls,), {'module_type': item})
        return cls._specialisations[item]


def this():
    """Return the module currently being constructed."""
    return Singleton.builder.current_module


def pin(*pins: Value) -> None:
    """Expose combinational pins from the current module being constructed."""
    module = Singleton.builder.current_module
    if module is None:
        raise RuntimeError("pin() must be called within an active module context")
    if not hasattr(module, 'pins') or module.pins is None:
        module.pins = []
    module.pins.extend(pins)


def _validate_outer_arguments(
    func: Callable[..., Any], args: tuple, kwargs: dict
) -> Dict[str, Any]:
    """Validate arguments passed to the outer factory function.

    Ensures runtime types align with annotations before proceeding to build
    the module. Returns a dictionary of validated arguments to feed into
    the outer function.
    """
    return validate_arguments(func, args, kwargs)


def _verify_inner_name(outer_name: str, inner_name: str) -> None:
    """Ensure the inner function follows the `<name>[_factory]` convention."""

    expected_inner_name = outer_name[:-8] if outer_name.endswith('_factory') else outer_name
    if inner_name != expected_inner_name:
        raise ValueError(
            f"Inner function name '{inner_name}' must match factory name '{expected_inner_name}'"
        )


def _rename_module(module: Any, inner_name: str) -> None:
    """Assign a unique, capitalised module name derived from `inner_name`."""

    unique_name = Singleton.builder.naming_manager.get_module_name(inner_name)
    if hasattr(module, 'name'):
        setattr(module, 'name', unique_name)


def _enter_module_context(module: Any) -> Block:
    """Initialise a module body and enter the builder context."""

    body = Block(Block.MODULE_ROOT)
    setattr(module, 'body', body)
    Singleton.builder.enter_context_of('module', module)
    return body


def _exit_module_context() -> None:
    """Exit the current module context."""

    Singleton.builder.exit_context_of('module')


def factory(
    module_type: Any
) -> Callable[[Callable[..., Callable[..., Any]]], Callable[..., Factory[Any]]]:
    """Universal factory decorator.

    The supplied `module_type` must provide `factory_check_signature(inner)` and
    `factory_create(inner, args)` helpers, responsible for type-specific
    validation and construction. This decorator handles the shared plumbing
    across all module kinds.
    """

    def decorator(func: Callable[..., Callable[..., Any]]) -> Callable[..., Factory[Any]]:
        @functools.wraps(func)
        def wrapper(*args, **kwargs) -> Factory[Any]:
            validated_args = _validate_outer_arguments(func, args, kwargs)
            inner = func(**validated_args)

            if not callable(inner):
                raise TypeError(
                    f"Factory function '{func.__name__}' must return a callable, "
                    f"got {type(inner).__name__}"
                )

            _verify_inner_name(func.__name__, inner.__name__)

            if not hasattr(module_type, 'factory_check_signature'):
                raise NotImplementedError(
                    f"{module_type} is missing 'factory_check_signature' implementation"
                )
            signature_payload = module_type.factory_check_signature(inner)

            if not hasattr(module_type, 'factory_create'):
                raise NotImplementedError(
                    f"{module_type} is missing 'factory_create' implementation"
                )
            create_result = module_type.factory_create(inner, signature_payload)
            if isinstance(create_result, tuple) and len(create_result) == 2:
                module_instance, inner_kwargs = create_result
            else:
                module_instance = create_result
                inner_kwargs = signature_payload if isinstance(signature_payload, dict) else {}

            _rename_module(module_instance, inner.__name__)

            body = _enter_module_context(module_instance)
            try:
                with body:
                    kwargs_for_inner = inner_kwargs or {}
                    if not isinstance(kwargs_for_inner, dict):
                        kwargs_for_inner = dict(kwargs_for_inner)
                    inner(**kwargs_for_inner)
            finally:
                _exit_module_context()

            specialised_factory = Factory[module_type]
            return specialised_factory(module_instance)

        return wrapper

    return decorator


__all__ = ['Factory', 'factory', 'this', 'pin']
