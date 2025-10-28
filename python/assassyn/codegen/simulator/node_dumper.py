"""Node reference dumper for simulator code generation."""

from .utils import int_imm_dumper_impl, fifo_name
from ...utils import unwrap_operand, namify
from ...ir.expr import Expr
from ...ir.array import Array
from ...ir.const import Const
from ...ir.module import Module, Port
from ...ir.expr.intrinsic import PureIntrinsic


def _handle_array(unwrapped, _module_ctx):
    """Handle Array nodes."""
    return namify(unwrapped.name)


def _handle_port(unwrapped, _module_ctx):
    """Handle Port nodes."""
    return fifo_name(unwrapped)


def _handle_const(unwrapped, _module_ctx):
    """Handle Const nodes."""
    return int_imm_dumper_impl(unwrapped.dtype, unwrapped.value)


def _handle_module(unwrapped, _module_ctx):
    """Handle Module nodes."""
    return namify(unwrapped.as_operand())


def _handle_expr(unwrapped, module_ctx):
    """Handle Expr nodes."""
    # Figure out the ID format based on context
    parent_module = getattr(unwrapped, 'parent', None)
    if module_ctx != parent_module:
        raw = namify(unwrapped.as_operand())
        field_id = f"{raw}_value"
        panic_log = f"Value {raw} invalid!"
        # Return as a block expression that evaluates to the value
        return f"""{{
                if let Some(x) = &sim.{field_id} {{
                    x
                }} else {{
                    panic!("{panic_log}");
                }}
            }}.clone()"""

    ref = namify(unwrapped.as_operand())
    if isinstance(unwrapped, PureIntrinsic) and unwrapped.opcode == PureIntrinsic.FIFO_PEEK:
        return f"{ref}.clone().unwrap()"

    dtype = unwrapped.dtype
    if dtype.bits <= 64:
        # Simple value
        return namify(ref)

    # Large value needs cloning
    return f"{ref}.clone()"


def _handle_str(unwrapped, _module_ctx):
    """Handle string nodes."""
    return f'"{unwrapped}"'


# Dispatch table mapping node types to their handler functions
_RVAL_HANDLER_DISPATCH = {
    Array: _handle_array,
    Port: _handle_port,
    Const: _handle_const,
    Module: _handle_module,
    Expr: _handle_expr,
    str: _handle_str,
}


def dump_rval_ref(module_ctx, node):
    """Dispatch to appropriate handler based on node kind."""
    unwrapped = unwrap_operand(node)
    node_type = type(unwrapped)

    # Try exact match first
    handler = _RVAL_HANDLER_DISPATCH.get(node_type)
    if handler is not None:
        return handler(unwrapped, module_ctx)

    # Fall back to isinstance check for subclasses
    for base_type, handler_func in _RVAL_HANDLER_DISPATCH.items():
        if isinstance(unwrapped, base_type):
            return handler_func(unwrapped, module_ctx)

    # Default case
    return namify(unwrapped.as_operand())
