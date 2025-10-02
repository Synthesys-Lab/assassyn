"""Factory decorator for pipeline stages.

This module provides the @pipeline.factory decorator to wrap a function
to be a pipeline stage factory.
"""

# pylint: disable=duplicate-code

import inspect
import functools
from typing import get_type_hints
from assassyn.ir.module import Port
from assassyn.ir.block import Block
from assassyn.builder import Singleton
from .stage import Stage


class StageFactory:  # pylint: disable=too-few-public-methods
    """A factory that builds a Stage object.

    This class wraps the inner function returned by a factory and provides
    a callable interface to build the Stage by growing its AST.

    Attributes:
        stage: The Stage object being built
    """

    def __init__(self, inner_func, ports, module_name):
        """Initialize the StageFactory.

        Args:
            inner_func: The inner function that grows the AST
            ports: Dictionary mapping port names to Port objects
            module_name: Name for the module
        """
        self._inner_func = inner_func
        self._ports = ports
        self._module_name = module_name
        # Create Stage object immediately so it can be accessed via .stage attribute
        self.stage = Stage(self._ports, self._module_name)

    def _build(self):
        """Build the Stage by calling the inner function and return its result."""
        # Set up module context and call inner function to grow AST
        self.stage.m.body = Block(Block.MODULE_ROOT)
        Singleton.builder.enter_context_of('module', self.stage.m)

        try:
            with self.stage.m.body:
                # Call inner function with the ports as arguments
                # Return whatever the inner function returns
                result = self._inner_func(**dict(self._ports))
        finally:
            Singleton.builder.exit_context_of('module')

        return result

    def __call__(self):
        """Callable wrapper to _build method."""
        return self._build()

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
        The current Module object from Singleton.builder.current_module.
    """
    return Singleton.builder.current_module


def factory(func):
    """Decorator to create a pipeline stage factory.

    This decorator:
    1. Enforces type checking on factory function arguments
    2. Validates the returned inner function is callable
    3. Returns a StageFactory that builds a Stage when called

    The returned StageFactory, when called, will:
    1. Extract Port[<type>] arguments from inner function signature
    2. Create a Stage wrapping a Module with those ports
    3. Call the inner function to grow the AST

    Args:
        func: The factory function to decorate. Must return a callable.

    Returns:
        A wrapper function that returns a StageFactory instance.
    """

    @functools.wraps(func)
    def wrapper(*args, **kwargs):  # pylint: disable=too-many-locals,too-many-branches
        # Step 1: Type check and unwrap arguments to the factory function
        sig = inspect.signature(func)
        bound_args = sig.bind(*args, **kwargs)
        bound_args.apply_defaults()

        # Get type hints for the factory function
        type_hints = get_type_hints(func)

        # Check each argument against its type annotation and unwrap StageFactory to Stage
        unwrapped_args = {}
        for param_name, param_value in bound_args.arguments.items():
            # Type check first, then unwrap if necessary
            if param_name in type_hints:
                expected_type = type_hints[param_name]
                # If type check fails, try to unwrap StageFactory to Stage
                if not isinstance(param_value, expected_type):
                    if isinstance(param_value, StageFactory) and expected_type == Stage:
                        unwrapped_value = param_value.stage
                        assert isinstance(unwrapped_value, Stage), (
                            f"StageFactory.stage must be a Stage, got {type(unwrapped_value)}"
                        )
                    else:
                        raise TypeError(
                            f"Argument '{param_name}' must be of type {expected_type}, "
                            f"got {type(param_value)}"
                        )
                else:
                    unwrapped_value = param_value
            else:
                unwrapped_value = param_value

            unwrapped_args[param_name] = unwrapped_value

        # Step 2: Call the factory function with unwrapped arguments to get the inner function
        inner_func = func(**unwrapped_args)

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

        # Step 5: Create and return StageFactory instance
        # Use naming manager to generate unique module name for multiple instantiations
        module_name = Singleton.naming_manager.get_module_name(inner_func.__name__)
        return StageFactory(inner_func, ports, module_name)

    return wrapper


__all__ = ['StageFactory', 'factory', 'pop_all', 'this', 'Stage']
