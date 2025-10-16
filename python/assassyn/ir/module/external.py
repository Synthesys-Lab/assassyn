'''External module implementation for integrating SystemVerilog modules.'''

from __future__ import annotations

# pylint: disable=duplicate-code,too-few-public-methods

from dataclasses import dataclass
from typing import Dict, Optional

from ...builder import Singleton
from ..dtype import DType
from ..block import Block
from ..expr import Expr, WireRead, wire_assign, wire_read
from ..visitor import Visitor
from .downstream import Downstream
from .module import Module, Wire




@dataclass(frozen=True)
class _ExternalConfig:
    '''Resolved configuration for an `ExternalSV` subclass.'''

    file_path: Optional[str]
    module_name: Optional[str]
    has_clock: bool
    has_reset: bool
    no_arbiter: bool
    in_wires: Dict[str, WireIn | WireOut | RegOut]
    out_wires: Dict[str, WireOut | RegOut]


class WireIn:
    '''Wrapper class for declaring ExternalSV wire inputs.'''

    def __init__(self, dtype: DType):
        if not isinstance(dtype, DType):
            raise TypeError("WireIn expects an assassyn dtype instance")
        self.dtype = dtype

    @classmethod
    def __class_getitem__(cls, dtype: DType):
        return cls(dtype)


class WireOut:
    '''Wrapper class for declaring ExternalSV combinational outputs.'''

    def __init__(self, dtype: DType):
        if not isinstance(dtype, DType):
            raise TypeError("WireOut expects an assassyn dtype instance")
        self.dtype = dtype

    @classmethod
    def __class_getitem__(cls, dtype: DType):
        return cls(dtype)


class RegOut:
    '''Wrapper class for declaring ExternalSV registered outputs.'''

    def __init__(self, dtype: DType):
        if not isinstance(dtype, DType):
            raise TypeError("RegOut expects an assassyn dtype instance")
        self.dtype = dtype

    @classmethod
    def __class_getitem__(cls, dtype: DType):
        return cls(dtype)


def _ensure_property(cls, name: str, direction: str):
    '''Install attribute helpers for accessing external wires.'''
    if hasattr(cls, name):
        return
    if direction == 'output':
        def getter(self, wire_name=name):
            return self.out_wires[wire_name]
        setattr(cls, name, property(getter))
    else:
        def getter(self, wire_name=name):
            return self.in_wires[wire_name]

        def setter(self, value, wire_name=name):
            self.in_wires[wire_name] = value
        setattr(cls, name, property(getter, setter))


def external(cls):
    '''Decorator that enables the simplified ExternalSV frontend.'''
    if not issubclass(cls, ExternalSV):
        raise TypeError("@external can only decorate ExternalSV subclasses")

    annotations = getattr(cls, '__annotations__', {})
    in_wires: Dict[str, DType] = {}
    out_wires: Dict[str, DType] = {}

    for name, annotation in annotations.items():
        if isinstance(annotation, WireIn):
            in_wires[name] = annotation
            _ensure_property(cls, name, 'input')
        elif isinstance(annotation, WireOut):
            out_wires[name] = annotation
            _ensure_property(cls, name, 'output')
        elif isinstance(annotation, RegOut):
            out_wires[name] = annotation
            _ensure_property(cls, name, 'output')

    file_path = getattr(cls, '__source__', None)
    module_name = getattr(cls, '__module_name__', None)
    has_clock = getattr(cls, '__has_clock__', False)
    has_reset = getattr(cls, '__has_reset__', False)
    no_arbiter = getattr(cls, '__no_arbiter__', False)

    cls.__external_config__ = _ExternalConfig(
        file_path=file_path,
        module_name=module_name,
        has_clock=bool(has_clock),
        has_reset=bool(has_reset),
        no_arbiter=bool(no_arbiter),
        in_wires=in_wires,
        out_wires=out_wires,
    )
    return cls


def _read_output_value(module: ExternalSV, wire: Wire):
    '''Deprecated helper retained for backward compatibility.'''
    return module.ensure_output_exposed(wire)


class _ExternalWireReadCollector(Visitor):
    '''Visitor that collects WireRead expressions attached to an ExternalSV module.'''

    def __init__(self, module: 'ExternalSV'):
        super().__init__()
        self._module = module
        self.reads: dict[Wire, WireRead] = {}

    def dispatch(self, node):
        if self.reads and len(self.reads) == len(self._module.declared_output_wires):
            return
        super().dispatch(node)

    def visit_block(self, node: Block):
        if self.reads and len(self.reads) == len(self._module.declared_output_wires):
            return
        super().visit_block(node)

    def visit_expr(self, node: Expr):
        if isinstance(node, WireRead):
            wire = node.wire
            owner = getattr(wire, 'module', None) or getattr(wire, 'parent', None)
            if owner is self._module and wire not in self.reads:
                self.reads[wire] = node
        for operand in getattr(node, 'operands', []):
            value = getattr(operand, 'value', operand)
            if isinstance(value, Expr):
                self.dispatch(value)


class _ExternalRegOutProxy:
    '''Lightweight proxy that exposes read-only register semantics via indexing.'''

    def __init__(self, module: ExternalSV, wire: Wire):
        self._module = module
        self._wire = wire

    def __getitem__(self, index):
        if isinstance(index, int):
            if index != 0:
                raise IndexError("External RegOut only supports index 0")
        else:
            raise TypeError("External RegOut expects an integer index (0)")
        return self._module._ensure_output_exposed(self._wire)

    @property
    def dtype(self):
        '''Return the dtype of the underlying register output.'''
        return self._wire.dtype

    def __repr__(self):
        module_name = getattr(self._module, 'name', type(self._module).__name__)
        wire_name = getattr(self._wire, 'name', '<unnamed>')
        return f'<RegOutProxy {module_name}.{wire_name}>'


class DirectionalWires:
    """Adapter exposing directional wire access consistent with the simplified API."""

    def __init__(self, ext_module, direction):
        self._module = ext_module
        self._direction = direction

    def _get_wire(self, key):
        wire = self._module.wires.get(key)
        if wire is None:
            raise KeyError(f"Wire '{key}' not found")
        if wire.direction != self._direction:
            raise ValueError(f"Wire '{key}' is not an {self._direction} wire")
        return wire

    def __contains__(self, key):
        wire = self._module.wires.get(key)
        return wire is not None and wire.direction == self._direction

    def __iter__(self):
        return iter(self.keys())

    def __getitem__(self, key):
        self._module._apply_pending_connections()
        wire = self._get_wire(key)
        if self._direction == 'output':
            expr = self._module.ensure_output_exposed(wire)
            if getattr(wire, 'kind', 'wire') == 'reg':
                return _ExternalRegOutProxy(self._module, wire)
            return expr
        return wire.value

    def __setitem__(self, key, value):
        if self._direction != 'input':
            raise ValueError(f"Cannot assign to '{key}' on output wires")
        wire = self._get_wire(key)
        wire_assign(wire, value)
        wire.assign(value)

    def keys(self):
        """Return the names of wires that match this adapter's direction."""
        return [
            name for name, wire in self._module.wires.items()
            if wire.direction == self._direction
        ]

    def items(self):
        """Iterate over (name, value) pairs for the selected direction."""
        for name, wire in self._module.wires.items():
            if wire.direction != self._direction:
                continue
            yield name, self[name]

    def values(self):
        """Iterate over wire values for the selected direction."""
        for _, value in self.items():
            yield value


class ExternalSV(Downstream):  # pylint: disable=too-many-instance-attributes
    '''An external block implemented in SystemVerilog.'''

    __external_config__: _ExternalConfig | None = None

    # pylint: disable=too-many-arguments,too-many-locals,too-many-branches,too-many-statements
    def __init__(
        self,
        file_path=None,
        in_wires=None,
        out_wires=None,
        module_name=None,
        no_arbiter=False,
        has_clock=False,
        has_reset=False,
        **wire_connections,
    ):
        '''Construct an external module.

        Args:
            file_path (str): Path to the SystemVerilog file containing the module.
            in_wires (dict, optional): Named input wire definitions `{name: dtype}`.
            out_wires (dict, optional): Named output wire definitions `{name: dtype}`.
            module_name (str, optional): Name of the module in the SystemVerilog file.
                                         Defaults to the class name.
            no_arbiter (bool): Whether to disable arbiter rewriting.
            **wire_connections: Optional initial assignments for declared input wires.
        '''
        config = getattr(type(self), '__external_config__', None)

        if config:
            in_wires = in_wires or config.in_wires
            out_wires = out_wires or config.out_wires
            file_path = file_path or config.file_path
            module_name = module_name or config.module_name
            has_clock = has_clock or config.has_clock
            has_reset = has_reset or config.has_reset
            no_arbiter = no_arbiter or config.no_arbiter

        if file_path is None:
            raise ValueError(
                "ExternalSV requires a 'file_path'. "
                "Provide it explicitly or use the @external decorator with '__source__'."
            )

        super().__init__()

        # Store external file information
        self.file_path = file_path

        self.external_module_name = module_name or type(self).__name__
        self.has_clock = has_clock
        self.has_reset = has_reset

        self._attrs = {}
        if no_arbiter:
            self._attrs[Module.ATTR_DISABLE_ARBITER] = True
        self._attrs[Module.ATTR_EXTERNAL] = True

        self._wires = {}
        self._exposed_output_reads = {}

        decl_in_wires = in_wires or {}
        decl_out_wires = out_wires or {}

        def _register_wire(name, dtype, direction, kind):
            wire = Wire(dtype, direction, self, kind=kind)
            wire.name = name
            self._wires[name] = wire

        self._declared_in_wires = decl_in_wires
        self._declared_out_wires = decl_out_wires

        for wire_name, decl in decl_in_wires.items():
            if not isinstance(decl, (WireIn, WireOut, RegOut)):
                raise TypeError(
                    f"Wire '{wire_name}' must be declared with WireIn/WireOut/RegOut, "
                    f"got {type(decl).__name__}"
                )
            kind = 'reg' if isinstance(decl, RegOut) else 'wire'
            _register_wire(wire_name, decl.dtype, 'input', kind)
        for wire_name, decl in decl_out_wires.items():
            if not isinstance(decl, (WireOut, RegOut)):
                raise TypeError(
                    f"Output wire '{wire_name}' must be declared with WireOut/RegOut, "
                    f"got {type(decl).__name__}"
                )
            kind = 'reg' if isinstance(decl, RegOut) else 'wire'
            _register_wire(wire_name, decl.dtype, 'output', kind)

        self.in_wires = DirectionalWires(self, 'input')
        self.out_wires = DirectionalWires(self, 'output')

        self._pending_wire_connections = None
        if wire_connections:
            validated = {}
            for wire_name, value in wire_connections.items():
                wire_obj = self._wires.get(wire_name)
                if wire_obj is None:
                    raise KeyError(
                        f"Cannot assign to undefined wire '{wire_name}'"
                    )
                if wire_obj.direction != 'input':
                    raise ValueError(
                        "Cannot assign to output wire "
                        f"'{wire_name}' during initialization"
                    )
                validated[wire_name] = value
            if validated:
                self._pending_wire_connections = validated

    def _apply_pending_connections(self):
        '''Apply any deferred constructor assignments when context is available.'''
        if not self._pending_wire_connections:
            return
        builder = Singleton.builder
        if builder is None or builder.current_module is None or builder.current_block is None:
            return
        assignments = self._pending_wire_connections
        self._pending_wire_connections = None
        for wire_name, value in assignments.items():
            wire_obj = self._wires[wire_name]
            wire_assign(wire_obj, value)
            wire_obj.assign(value)

    def _ensure_output_exposed(self, wire: Wire):
        '''Guarantee a WireRead exists in this module for the given output wire.'''
        if wire in self._exposed_output_reads:
            return self._exposed_output_reads[wire]

        self._harvest_output_reads()
        if wire in self._exposed_output_reads:
            return self._exposed_output_reads[wire]

        builder = Singleton.builder
        if builder is None:
            raise RuntimeError(
                "External wire access requires an active SysBuilder context"
            )

        need_context = (
            builder.current_module is not self
            or builder.current_block is None
            or getattr(builder.current_block, 'module', None) is not self
        )

        if self.body is None:
            self.body = Block(Block.MODULE_ROOT)
            self.body.parent = self
            self.body.module = self

        if need_context:
            builder.enter_context_of('module', self)
            builder.enter_context_of('block', self.body)

        try:
            expr = wire_read(wire)
        finally:
            if need_context:
                builder.exit_context_of('block')
                builder.exit_context_of('module')

        self._exposed_output_reads[wire] = expr
        return expr

    def _harvest_output_reads(self):
        '''Populate cached output WireRead expressions using the visitor walker.'''
        if not self._declared_out_wires or self.body is None:
            return
        collector = _ExternalWireReadCollector(self)
        collector.visit_block(self.body)
        for wire, expr in collector.reads.items():
            self._exposed_output_reads.setdefault(wire, expr)

    def ensure_output_exposed(self, wire: Wire):
        '''Public wrapper for exposing output wires as expressions.'''
        return self._ensure_output_exposed(wire)

    @property
    def declared_output_wires(self):
        '''Return declared output wires metadata.'''
        return self._declared_out_wires

    @property
    def wires(self):
        """Expose declared wires keyed by name for helper adapters."""
        return self._wires

    def __setitem__(self, key, value):
        '''Allow assignment to wires using bracket notation.'''
        if key in self.in_wires:
            self.in_wires[key] = value
            return
        raise KeyError(f"Wire '{key}' not found")

    def __getitem__(self, key):
        '''Allow access to wires using bracket notation.'''
        if key in self.out_wires:
            return self.out_wires[key]
        if key in self.in_wires:
            return self.in_wires[key]
        raise KeyError(f"Wire '{key}' not found")

    def in_assign(self, **kwargs):
        '''Assign values to input wires using keyword arguments.

        Args:
            **kwargs: Wire name to value mappings (e.g., a=value, b=value)
        '''
        builder = Singleton.builder
        if builder is None:
            raise RuntimeError(
                "ExternalSV.in_assign requires an active SysBuilder context"
            )

        self._apply_pending_connections()

        if self.body is None:
            self.body = Block(Block.MODULE_ROOT)
            self.body.parent = self
            self.body.module = self

        builder.enter_context_of('module', self)
        builder.enter_context_of('block', self.body)

        try:
            for wire_name, value in kwargs.items():
                self.in_wires[wire_name] = value

            outputs = [self.out_wires[name] for name in self.out_wires]
        finally:
            builder.exit_context_of('block')
            builder.exit_context_of('module')

        if not outputs:
            return None
        if len(outputs) == 1:
            return outputs[0]
        return tuple(outputs)

    def __repr__(self):
        '''String representation of the external module.'''
        wires = '\n    '.join(
            f"{name}: {wire}" for name, wire in self._wires.items()
        )
        wire_lines = f'{{\n    {wires}\n  }} ' if wires else ''
        attrs = ', '.join(
            f'{Module.MODULE_ATTR_STR[i]}: {j}' for i, j in self._attrs.items()
        )
        attrs = f'#[{attrs}] ' if attrs else ''
        var_id = self.as_operand()

        ext_info = f'  // External file: {self.file_path}\n'
        ext_info += f'  // External module name: {self.external_module_name}\n'

        Singleton.repr_ident = 2
        body = self.body.__repr__() if self.body else ''
        ext = self._dump_externals()
        return f'''{ext}{ext_info}  {attrs}
  {var_id} = external_module {self.name} {wire_lines}{{
{body}
  }}'''
