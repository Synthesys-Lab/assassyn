"""Rvalue dumping utilities for Verilog code generation."""

from ...ir.module import Module, Port, Wire
from ...ir.const import Const
from ...ir.array import Array
from ...ir.dtype import RecordValue
from ...ir.expr import Expr, FIFOPop
from ...utils import namify, unwrap_operand
from .utils import dump_type


# pylint: disable=too-many-return-statements,too-many-branches
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
    if (
        isinstance(node, Expr)
        and dumper.current_module is not None
        and hasattr(dumper.current_module, 'externals')
        and node in dumper.current_module.externals
        and not dumper.is_top_generation
    ):
        return f"self.{dumper.get_external_port_name(node)}"
    if isinstance(node, Module):
        return namify(node.name)
    if isinstance(node, Array):
        array = node
        return namify(array.name)
    if isinstance(node, Port):
        return namify(node.name)
    if isinstance(node, FIFOPop):
        if not with_namespace:
            return f'self.{namify(node.fifo.name)}'
        return namify(node.fifo.module.name) + "_" + namify(node.fifo.name)
    if isinstance(node, Const):
        int_imm = node
        value = int_imm.value
        ty = dump_type(int_imm.dtype)
        return f"{ty}({value})"
    if isinstance(node, str):
        value = node
        return f'"{value}"'
    if isinstance(node, Expr):
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

    if isinstance(node, RecordValue):
        return dump_rval(dumper, node.value(), with_namespace, module_name)
    if isinstance(node, Wire):
        # For wires, we use their name directly
        return namify(node.name)

    raise ValueError(f"Unknown node of kind {type(node).__name__}")
