"""Factory decorator for pipeline stages.

This module provides the @pipeline.factory decorator to wrap a function
to be a pipeline stage factory.
"""

import inspect
import functools
from typing import get_type_hints
from assassyn.ir.module import Port
from assassyn.ir.block import Block
from assassyn.builder import Singleton
from .stage import Stage


def pop_all(validate=True):
    """Pop all ports from the current module.

    Args:
        validate: If True, validates all ports before popping.

    Returns:
        The popped values from all ports.
    """
    module = Singleton.builder.current_module
    assert module is not None, "pop_all must be called within a module context"
    return module.pop_all_ports(validate)


def this():
    """Return the current module being built.

    Returns:
        The current Module object from Singleton.builder.current_module().
    """
    return Singleton.builder.current_module()


def factory(func):
    """Decorator to create a pipeline stage factory.

    This decorator:
    1. Enforces type checking on factory function arguments
    2. Validates the returned inner function is callable
    3. Extracts Port[<type>] arguments from inner function
    4. Creates a Stage wrapping a Module with those ports
    5. Calls the inner function to grow the AST

    Args:
        func: The factory function to decorate. Must return a callable.

    Returns:
        A wrapper function that creates and returns a Stage object.
    """

    @functools.wraps(func)
    def wrapper(*args, **kwargs):  # pylint: disable=too-many-locals
        # Step 1: Type check all arguments to the factory function
        sig = inspect.signature(func)
        bound_args = sig.bind(*args, **kwargs)
        bound_args.apply_defaults()

        # Get type hints for the factory function
        type_hints = get_type_hints(func)

        # Check each argument against its type annotation
        for param_name, param_value in bound_args.arguments.items():
            if param_name in type_hints:
                expected_type = type_hints[param_name]
                if not isinstance(param_value, expected_type):
                    raise TypeError(
                        f"Argument '{param_name}' must be of type {expected_type}, "
                        f"got {type(param_value)}"
                    )

        # Step 2: Call the factory function to get the inner function
        inner_func = func(*args, **kwargs)

        if not callable(inner_func):
            raise TypeError(
                f"Factory function '{func.__name__}' must return a callable, "
                f"got {type(inner_func)}"
            )

        # Step 3: Extract and validate inner function signature
        inner_sig = inspect.signature(inner_func)
        inner_hints = get_type_hints(inner_func)

        # Build ports dictionary
        ports = {}
        for param_name in inner_sig.parameters:
            # Check that all parameters have type annotations
            if param_name not in inner_hints:
                raise TypeError(
                    f"Parameter '{param_name}' in '{inner_func.__name__}' "
                    f"must have a type annotation"
                )

            param_type = inner_hints[param_name]

            # Check if the annotation is already a Port instance (from Port[UInt(32)])
            if isinstance(param_type, Port):
                # Port.__class_getitem__ already created the Port instance for us
                ports[param_name] = param_type
            else:
                raise TypeError(
                    f"Parameter '{param_name}' must be annotated as Port[<DataType>], "
                    f"got {param_type}"
                )

        # Step 4: Verify naming convention
        # Inner function name should match factory name with '_factory' suffix removed
        expected_inner_name = func.__name__
        if expected_inner_name.endswith('_factory'):
            expected_inner_name = expected_inner_name[:-8]  # Remove '_factory'

        if inner_func.__name__ != expected_inner_name:
            raise ValueError(
                f"Inner function name '{inner_func.__name__}' must match "
                f"factory name '{expected_inner_name}' (derived from '{func.__name__}')"
            )

        # Step 5: Create Stage object with capitalized module name
        # Special case: 'driver' becomes 'Driver' for backward compatibility
        module_name = inner_func.__name__.capitalize()
        stage = Stage(ports, module_name)

        # Step 6: Set up module context and call inner function to grow AST
        stage.m.body = Block(Block.MODULE_ROOT)
        Singleton.builder.enter_context_of('module', stage.m)

        try:
            with stage.m.body:
                # Call inner function with the ports as arguments
                inner_func(**dict(ports))
        finally:
            Singleton.builder.exit_context_of('module')

        return stage

    return wrapper
