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
        if self.bind is None:
            # Create a new Bind node on first call
            self.bind = Bind(self.m, args)
        else:
            # Update the existing bind with new arguments
            self.bind.update_value(args)
        return self

    def __call__(self):
        """Create an async call to the bind.

        This serves a similar purpose to Module.async_called in the old frontend.
        Calls are always void argument as arguments are fed by bindings.
        """
        if self.bind is None:
            raise RuntimeError(
                f"Cannot call stage '{self.m.name}' without binding arguments first. "
                f"Use 'stage << args' to bind arguments before calling."
            )
        # Create async call to the bind
        self.m.async_called(self.bind)
