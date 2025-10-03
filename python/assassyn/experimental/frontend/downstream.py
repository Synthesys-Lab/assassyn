"""Downstream-specific helpers for the unified experimental frontend factory."""

from __future__ import annotations

import inspect
from typing import Any, Callable

from assassyn.ir.module.downstream import Downstream


def factory_check_signature(inner: Callable[..., Any]) -> bool:
    """Validate that the inner function has no arguments (empty signature)."""

    signature = inspect.signature(inner)
    if len(signature.parameters) > 0:
        raise TypeError(
            f"Downstream inner function '{inner.__name__}' must have no arguments, "
            f"but found: {list(signature.parameters.keys())}"
        )
    return True


def factory_create(_inner: Callable[..., Any], _args: bool) -> Downstream:
    """Instantiate a Downstream module."""

    module = Downstream()
    return module


# Register the special factory hooks on Downstream so the generic decorator can call them.
Downstream.factory_check_signature = staticmethod(factory_check_signature)
Downstream.factory_create = staticmethod(factory_create)


__all__ = ['factory_check_signature', 'factory_create']
