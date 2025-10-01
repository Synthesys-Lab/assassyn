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
        from assassyn.ir.value import Value

        # Convert single Value to tuple
        if isinstance(args, Value):
            args = (args,)

        # Create empty bind if it doesn't exist
        if self.bind is None:
            self.bind = self.m.bind()

        # Convert args to kwargs
        if isinstance(args, dict):
            kwargs = args
        else:
            # Tuple binding - convert positional to keyword args
            # Find unbound ports by traversing self.bind.pushes
            all_port_names = [port.name for port in self.m.ports]
            bound_port_names = set(push.fifo.name for push in self.bind.pushes)
            unbound_ports = [name for name in all_port_names if name not in bound_port_names]

            if len(args) > len(unbound_ports):
                raise ValueError(f"Too many arguments: {len(args)} provided but only {len(unbound_ports)} ports unbound")

            # Map positional args to unbound ports
            kwargs = dict(zip(unbound_ports[:len(args)], args))

        # Bind the arguments
        self.bind.bind(**kwargs)

        return self

    def __call__(self):
        """Create an async call to the bind.

        This serves a similar purpose to Module.async_called in the old frontend.
        Calls are always void argument as arguments are fed by bindings.
        """
        if self.bind is None:
            raise ValueError("Cannot call stage without binding arguments first")

        return self.bind.async_called()
