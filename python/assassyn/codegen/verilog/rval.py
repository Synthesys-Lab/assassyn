"""Rvalue dumping utilities for Verilog code generation."""

from ...ir.module import Module, Port, Wire
from ...ir.const import Const
from ...ir.array import Array
from ...ir.dtype import RecordValue
from ...ir.expr import Expr, FIFOPop
from ...utils import namify, unwrap_operand
from .utils import dump_type
def _dump_fifo_pop(_dumper, node, with_namespace: bool, _module_name: str = None) -> str:
    if not with_namespace:
        return f'self.{namify(node.fifo.name)}'
    return namify(node.fifo.module.name) + "_" + namify(node.fifo.name)


def _dump_const(_dumper, node, _with_namespace: bool, _module_name: str = None) -> str:
    value = node.value
    ty = dump_type(node.dtype)
    return f"{ty}({value})"


def _dump_str(_dumper, node, _with_namespace: bool, _module_name: str = None) -> str:
    return f'"{node}"'


def _dump_expr(dumper, node, with_namespace: bool, module_name: str = None) -> str:
    if node not in dumper.expr_to_name:
        base_name = namify(node.as_operand())
        # Handle anonymous expressions which namify to '_' or an empty string.
        if not base_name or base_name == '_':
            base_name = 'tmp'

        count = dumper.name_counters[base_name]
        unique_name = f"{base_name}_{count}" if count > 0 else base_name
        dumper.name_counters[base_name] += 1
        dumper.expr_to_name[node] = unique_name

    unique_name = dumper.expr_to_name[node]

    if with_namespace:
        owner_module_name = namify(node.parent.module.name)
        if owner_module_name is None:
            owner_module_name = module_name
        return f"{owner_module_name}_{unique_name}"
    return unique_name


def _dump_record_value(dumper, node, with_namespace: bool, module_name: str = None) -> str:
    return dump_rval(dumper, node.value(), with_namespace, module_name)
# Dispatch table mapping node types to their dump functions
_RVAL_DUMP_DISPATCH = {
    Module: lambda _dumper, node, _with_namespace, _module_name=None: namify(node.name),
    Array: lambda _dumper, node, _with_namespace, _module_name=None: namify(node.name),
    Port: lambda _dumper, node, _with_namespace, _module_name=None: namify(node.name),
    FIFOPop: _dump_fifo_pop,
    Const: _dump_const,
    str: _dump_str,
    RecordValue: _dump_record_value,
    Wire: lambda _dumper, node, _with_namespace, _module_name=None: namify(node.name),
}


def dump_rval(dumper, node, with_namespace: bool, module_name: str = None) -> str:
    """Dump a reference to a node with options.

    Args:
        dumper: The CIRCTDumper instance
        node: The node to dump
        with_namespace: Whether to include namespace in the name
        module_name: Optional module name to use

    Returns:
        String representation of the rvalue
    """
    node = unwrap_operand(node)

    # Special case: check for external expressions first
    if (
        isinstance(node, Expr)
        and dumper.current_module is not None
        and hasattr(dumper.current_module, 'externals')
        and node in dumper.current_module.externals
        and not dumper.is_top_generation
    ):
        return f"self.{dumper.get_external_port_name(node)}"

    node_type = type(node)

    # Try exact match first
    dump_func = _RVAL_DUMP_DISPATCH.get(node_type)
    if dump_func is not None:
        return dump_func(dumper, node, with_namespace, module_name)

    # Handle Expr subclasses (must check after other types since some inherit from Expr)
    if isinstance(node, Expr):
        return _dump_expr(dumper, node, with_namespace, module_name)

    # Fall back to isinstance check for other subclasses
    for base_type, func in _RVAL_DUMP_DISPATCH.items():
        if isinstance(node, base_type):
            return func(dumper, node, with_namespace, module_name)

    raise ValueError(f"Unknown node of kind {type(node).__name__}")
