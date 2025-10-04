"""Module-specific helpers for the unified experimental frontend factory."""

from __future__ import annotations

import inspect
from typing import TYPE_CHECKING, Any, Callable, Dict, Optional, Tuple, get_type_hints

from assassyn.builder import Singleton
from assassyn.ir.dtype import DType
from assassyn.ir.module import Module, Port
from assassyn.ir.value import Value

from .factory import Factory

if TYPE_CHECKING:
    from assassyn.ir.expr import Bind


def factory_check_signature(inner: Callable[..., Any]) -> Dict[str, Port]:
    """Validate inner signature and synthesise module ports."""

    signature = inspect.signature(inner)
    annotations = get_type_hints(inner)
    ports: Dict[str, Port] = {}

    for param_name in signature.parameters:
        annotation = annotations.get(param_name)
        if annotation is None:
            raise TypeError(
                f"Parameter '{param_name}' in '{inner.__name__}' must have a type annotation"
            )
        if not isinstance(annotation, Port):
            raise TypeError(
                f"Parameter '{param_name}' must be annotated as "
                f"Port[<DataType>], got {annotation!r}"
            )
        if not isinstance(annotation.dtype, DType):
            raise TypeError(
                f"Port '{param_name}' must wrap a DType, got {type(annotation.dtype).__name__}"
            )
        ports[param_name] = Port(annotation.dtype)

    return ports


def factory_create(
    _inner: Callable[..., Any], args: Dict[str, Port]
) -> Tuple[Module, Dict[str, Port]]:
    """Instantiate a Module and prepare kwargs for the inner builder."""

    module = Module(args)
    inner_kwargs = {name: getattr(module, name) for name in args}
    return module, inner_kwargs


# Register the special factory hooks on Module so the generic decorator can call them.
Module.factory_check_signature = staticmethod(factory_check_signature)
Module.factory_create = staticmethod(factory_create)


def pop_all(validate: bool = False):
    """Pop all ports from the current module under construction."""

    module = Singleton.builder.current_module
    if module is None:
        raise RuntimeError("pop_all must be called within an active module context")
    return module.pop_all_ports(validate)


class ModuleFactory(Factory[Module]):
    """Wrapper around `Module` providing bind/call sugar."""

    module_type = Module

    def __init__(self, module: Module):
        super().__init__(module)
        self.bind: Optional['Bind'] = None

    def __lshift__(self, args):
        if isinstance(args, Value):
            args = (args,)
        if self.bind is None:
            self.bind = self.module.bind()

        if isinstance(args, dict):
            kwargs = args
        elif isinstance(args, tuple):
            all_ports = [port.name for port in self.module.ports]
            bound_ports = {push.fifo.name for push in self.bind.pushes}
            unbound_ports = [name for name in all_ports if name not in bound_ports]
            if len(args) > len(unbound_ports):
                raise ValueError(
                    f"Too many positional arguments: {len(args)} provided but only "
                    f"{len(unbound_ports)} ports unbound"
                )
            kwargs = dict(zip(unbound_ports, args))
        else:
            raise TypeError("Arguments to '<<' must be Value, tuple, or dict")

        self.bind.bind(**kwargs)
        return self

    def __call__(self):
        if self.bind is None:
            raise ValueError("Cannot call module without binding arguments first")
        return self.bind.async_called()


Factory._specialisations[Module] = ModuleFactory  # pylint: disable=protected-access


__all__ = ['ModuleFactory', 'factory_check_signature', 'factory_create', 'pop_all']
