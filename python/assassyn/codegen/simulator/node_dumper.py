"""Node reference dumper for simulator code generation."""

from .utils import int_imm_dumper_impl, fifo_name
from ...utils import unwrap_operand, namify
from ...ir.expr import Expr
from ...ir.array import Array
from ...ir.const import Const
from ...ir.module import Module, Port
from ...ir.expr import PureIntrinsic

def dump_rval_ref( # pylint: disable=too-many-branches, too-many-return-statements
        module_ctx, _, node):
    """Dispatch to appropriate handler based on node kind."""

    unwrapped = unwrap_operand(node)

    if isinstance(unwrapped, Array):
        return namify(unwrapped.name)

    if isinstance(unwrapped, Port):
        return fifo_name(unwrapped)

    if isinstance(unwrapped, Const):
        return int_imm_dumper_impl(unwrapped.dtype, unwrapped.value)

    if isinstance(unwrapped, Module):
        return namify(unwrapped.as_operand())

    if isinstance(unwrapped, Expr):
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

    if isinstance(unwrapped, str):
        return f'"{unwrapped}"'

    # Default case
    return namify(unwrapped.as_operand())
