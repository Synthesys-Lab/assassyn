"""Stage - Pipeline Stage wrapper for Module objects.

A Stage wraps a Module object and provides a convenient interface
for binding arguments and making async calls.
"""

from assassyn.ir.module import Module, Port
from assassyn.ir.expr import Bind


class Stage:
    """A pipeline stage that wraps a Module object.

    Attributes:
        m: The wrapped Module object
        bind: The Bind node for argument binding
    """

    def __init__(self, module: dict[str, Port], name: str):
        """Initialize a Stage with ports and a name.

        Args:
            module: Dictionary mapping port names to Port objects
            name: Name for the stage
        """
        # Create the wrapped Module object with the given ports
        self.m = Module(module)
        # Rename the module to the given name
        self.m.name = name
        # Initialize bind as None (will be created on first __lshift__ call)
        self.bind = None

    def __lshift__(self, args: tuple | dict):
        """Bind arguments to the stage using the << operator.

        Args:
            args: Either a tuple (positional args) or dict (named args)
        """
        # TODO
        pass

    def __call__(self):
        """Create an async call to the bind.

        This serves a similar purpose to Module.async_called in the old frontend.
        Calls are always void argument as arguments are fed by bindings.
        """
        # TODO
        pass
