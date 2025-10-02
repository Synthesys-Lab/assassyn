"""Convergent downstream combinational logic factory decorator.

This module provides the @converge decorator to create downstream
combinational logic that converges across multiple modules.
"""

# pylint: disable=duplicate-code

import inspect
import functools
from assassyn.ir.module.downstream import Downstream
from assassyn.ir.block import Block
from assassyn.builder import Singleton


def converge(func):
    """Decorator to create a downstream combinational logic factory.

    This decorator creates convergent downstream combinational logic that
    serves a similar purpose as @downstream.combinational but is designed
    specifically for convergence across multiple modules.

    The decorator enforces:
    1. The inner function must have the same name as the outer function
       with '_factory' suffix removed
    2. The inner function should be returned by the outer function
    3. The inner function should not have any arguments

    Args:
        func: The factory function to decorate. Must return a callable.

    Returns:
        A wrapper function that creates and returns a Downstream object.

    Example:
        @converge
        def downstream_factory(a: Value, b: Value) -> Downstream:
            def downstream():
                c = a + b
                log("Downstream: {} + {} = {}", a, b, c)
            return downstream
    """

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        # Step 1: Call the factory function to get the inner function
        inner_func = func(*args, **kwargs)

        if not callable(inner_func):
            raise TypeError(
                f"Factory function '{func.__name__}' must return a callable, "
                f"got {type(inner_func)}"
            )

        # Step 2: Validate inner function signature
        inner_sig = inspect.signature(inner_func)

        # The inner function should not have any arguments
        if len(inner_sig.parameters) > 0:
            raise TypeError(
                f"Inner function '{inner_func.__name__}' should not have any arguments, "
                f"got {len(inner_sig.parameters)} parameter(s)"
            )

        # Step 3: Verify naming convention
        # Inner function name should match factory name with '_factory' suffix removed
        expected_inner_name = func.__name__
        if expected_inner_name.endswith('_factory'):
            expected_inner_name = expected_inner_name[:-8]  # Remove '_factory'

        if inner_func.__name__ != expected_inner_name:
            raise ValueError(
                f"Inner function name '{inner_func.__name__}' must match "
                f"factory name '{expected_inner_name}' (derived from '{func.__name__}')"
            )

        # Step 4: Create Downstream object
        # Use naming manager to generate unique module name for multiple instantiations
        downstream_class_name = Singleton.naming_manager.get_module_name(inner_func.__name__)

        # Dynamically create a Downstream subclass
        downstream_class = type(downstream_class_name, (Downstream,), {})

        # Instantiate the downstream module
        downstream_instance = downstream_class()

        # Step 5: Set up module context and call inner function to grow AST
        body = Block(Block.MODULE_ROOT)
        downstream_instance.body = body
        Singleton.builder.enter_context_of('module', downstream_instance)

        try:
            with body:
                # Call inner function to grow the AST
                inner_func()
        finally:
            Singleton.builder.exit_context_of('module')

        return downstream_instance

    return wrapper
