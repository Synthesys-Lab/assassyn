'''External module implementation for integrating SystemVerilog modules.'''

from __future__ import annotations

# pylint: disable=duplicate-code,too-few-public-methods

from dataclasses import dataclass
from os import fspath
from typing import Any, Dict, Generic, TypeVar, Literal


T = TypeVar('T')


def _normalize_external_parameters(parameters: Any) -> Dict[str, Any]:
    """Normalize ExternalSV parameter overrides into a plain dict.

    Accepted parameter values:
      - int/bool (bool coerced to int)
      - str
      - PathLike (converted via os.fspath)
    """
    if parameters is None:
        return {}
    if not isinstance(parameters, dict):
        raise TypeError("ExternalSV __parameters__ must be a dict of name -> value")

    normalized: Dict[str, Any] = {}
    for name, value in parameters.items():
        if not isinstance(name, str):
            raise TypeError("ExternalSV parameter names must be strings")
        if isinstance(value, bool):
            normalized[name] = int(value)
            continue
        if isinstance(value, int):
            normalized[name] = value
            continue
        if hasattr(value, "__fspath__"):
            normalized[name] = fspath(value)
            continue
        if isinstance(value, str):
            normalized[name] = value
            continue
        raise TypeError(
            "ExternalSV parameter values must be int/bool/str or PathLike"
        )
    return normalized


def _create_external_intrinsic(cls, **input_connections):
    """Factory function to create ExternalIntrinsic with proper IR builder tracking."""
    # pylint: disable=import-outside-toplevel
    from ...builder import ir_builder
    from ..expr.intrinsic import ExternalIntrinsic

    @ir_builder
    def _create():
        return ExternalIntrinsic(cls, **input_connections)

    return _create()


class ExternalSVMeta(type):
    """Metaclass for ExternalSV that makes the class callable to create intrinsics."""

    def __call__(cls, **input_connections):
        """When ExternalAdder(...) is called, create ExternalIntrinsic instead of instance."""
        return _create_external_intrinsic(cls, **input_connections)


@dataclass
class WireSpec:
    """Specification for an external module port."""
    name: str
    dtype: type  # DType instance
    direction: Literal['in', 'out']
    kind: Literal['wire', 'reg']  # 'reg' only for outputs


class WireIn(Generic[T]):
    """Type marker for input wire ports.

    Usage:
        a: WireIn[UInt(32)]
    """


class WireOut(Generic[T]):
    """Type marker for combinational output wire ports.

    Usage:
        c: WireOut[UInt(32)]
    """


class RegOut(Generic[T]):
    """Type marker for registered output ports (arrays).

    Usage:
        reg_out: RegOut[Bits(32)]
    """


class _ExternalRegOutProxy:
    """Proxy for RegOut array access that creates PureIntrinsic reads.

    This provides array-like indexing for registered outputs from external modules.
    """

    def __init__(self, external_intrinsic, port_name: str, dtype):
        self._external_intrinsic = external_intrinsic
        self._port_name = port_name
        self._dtype = dtype

    def __getitem__(self, index):
        # pylint: disable=import-outside-toplevel
        from ...builder import ir_builder
        from ..expr.intrinsic import PureIntrinsic
        from ..const import Const
        from ..dtype import UInt

        # Wrap index in Const if it's a Python int
        if isinstance(index, int):
            # Create a UInt constant for the index
            index_const = Const(UInt(32), index)
        else:
            index_const = index

        @ir_builder
        def _read():
            return PureIntrinsic(
                PureIntrinsic.EXTERNAL_OUTPUT_READ,
                self._external_intrinsic,
                self._port_name,
                index_const
            )
        return _read()

    @property
    def dtype(self):
        """Return the dtype of the underlying register output."""
        return self._dtype

    def __repr__(self):
        inst_name = getattr(self._external_intrinsic, '_external_class', '<unknown>').__name__
        return f'<RegOutProxy {inst_name}.{self._port_name}>'


def external(cls):
    """Decorator for ExternalSV subclasses.

    Validates annotations and creates _wires metadata.

    Usage:
        @external
        class MyExternal(ExternalSV):
            a: WireIn[UInt(32)]
            b: WireOut[UInt(32)]
            __source__ = "path/to/file.sv"
            __module_name__ = "my_module"
    """
    if not issubclass(cls, ExternalSV):
        raise TypeError("@external can only decorate ExternalSV subclasses")

    # Parse annotations to build _wires dict
    annotations = getattr(cls, '__annotations__', {})
    wires = {}

    for name, annotation in annotations.items():
        if name.startswith('__'):
            continue

        # Handle Generic type annotations
        origin = getattr(annotation, '__origin__', None)
        if origin is None:
            continue

        # Get the dtype from Generic args
        args = getattr(annotation, '__args__', ())
        if not args:
            continue
        dtype = args[0]

        if origin is WireIn:
            wires[name] = WireSpec(name, dtype, 'in', 'wire')
        elif origin is WireOut:
            wires[name] = WireSpec(name, dtype, 'out', 'wire')
        elif origin is RegOut:
            wires[name] = WireSpec(name, dtype, 'out', 'reg')

    cls.set_port_specs(wires)

    # Extract metadata
    cls.set_metadata({
        'source': getattr(cls, '__source__', None),
        'module_name': getattr(cls, '__module_name__', cls.__name__),
        'has_clock': getattr(cls, '__has_clock__', False),
        'has_reset': getattr(cls, '__has_reset__', False),
        'parameters': _normalize_external_parameters(getattr(cls, '__parameters__', None)),
    })

    return cls


class ExternalSV(metaclass=ExternalSVMeta):
    """Metadata descriptor for external SystemVerilog modules.

    This is not a Module or Downstream - calling the CLASS creates an ExternalIntrinsic.
    The metaclass makes ExternalAdder(...) create an intrinsic instead of an instance.

    Usage:
        @external
        class ExternalAdder(ExternalSV):
            a: WireIn[UInt(32)]
            b: WireIn[UInt(32)]
            c: WireOut[UInt(32)]
            __source__ = "path/to/adder.sv"
            __module_name__ = "adder"

        # Later in a module:
        result = ExternalAdder(a=value_a, b=value_b)  # Creates ExternalIntrinsic
        output = result.c  # Access output
    """
    _wires: Dict[str, WireSpec] = {}
    _metadata: Dict[str, Any] = {}

    @classmethod
    def set_port_specs(cls, wires: Dict[str, WireSpec]) -> None:
        """Store the port specification table."""
        cls._wires = wires

    @classmethod
    def set_metadata(cls, metadata: Dict[str, Any]) -> None:
        """Store metadata for the external module."""
        cls._metadata = metadata

    @classmethod
    def port_specs(cls) -> Dict[str, WireSpec]:
        """Return the registered port specifications."""
        return cls._wires

    @classmethod
    def metadata(cls) -> Dict[str, Any]:
        """Return metadata dictionary for the external module."""
        return cls._metadata

    @classmethod
    def parameters(cls) -> Dict[str, Any]:
        """Return a copy of the normalized external parameter overrides."""
        params = cls._metadata.get("parameters", {})
        return dict(params)
