"""Node reference dumper for simulator code generation."""

from .utils import int_imm_dumper_impl, fifo_name, dtype_to_rust_type
from .external import external_handle_field
from ...utils import unwrap_operand, namify
from ...ir.expr import Expr, WireRead
from ...ir.array import Array
from ...ir.const import Const
from ...ir.module import Module, Port
from ...ir.expr.intrinsic import PureIntrinsic
from ...ir.module.external import ExternalSV


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
    if isinstance(unwrapped, WireRead):
        wire = unwrapped.wire
        owner = getattr(wire, "parent", None) or getattr(wire, "module", None)
        if isinstance(owner, ExternalSV):
            kind = getattr(wire, "kind", "wire")
            if kind in ("wire", "reg"):
                field_id = namify(unwrapped.as_operand())
                handle_field = external_handle_field(owner.name)
                method_suffix = namify(wire.name)
                rust_ty = dtype_to_rust_type(unwrapped.dtype)
                return (
                    "{\n"
                    f"        if sim.{field_id}_value.is_none() {{\n"
                    f"            sim.{handle_field}.eval();\n"
                    f"            let value = sim.{handle_field}.get_{method_suffix}();\n"
                    f"            let value = ValueCastTo::<{rust_ty}>::cast(&value);\n"
                    f"            sim.{field_id}_value = Some(value.clone());\n"
                    "        }\n"
                    f"        sim.{field_id}_value.as_ref().unwrap().clone()\n"
                    "    }"
                )

    # Figure out the ID format based on context
    parent_block = unwrapped.parent
    if module_ctx != parent_block.module:
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
