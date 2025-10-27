"""Intrinsic code generation helpers for simulator.

This module contains helper functions to generate simulator code for intrinsic operations.
"""

# pylint: disable=too-many-locals, unused-argument
# pylint: disable=import-outside-toplevel

from ....ir.expr.intrinsic import PureIntrinsic, Intrinsic, ExternalIntrinsic
from ....utils import namify
from ..node_dumper import dump_rval_ref


def _codegen_fifo_peek(node, module_ctx):
    """Generate code for FIFO_PEEK intrinsic."""
    port_self = dump_rval_ref(module_ctx, node.get_operand(0))
    return f"sim.{port_self}.front().cloned()"


def _codegen_fifo_valid(node, module_ctx):
    """Generate code for FIFO_VALID intrinsic."""
    port_self = dump_rval_ref(module_ctx, node.get_operand(0))
    return f"!sim.{port_self}.is_empty()"


def _codegen_value_valid(node, module_ctx):
    """Generate code for VALUE_VALID intrinsic."""
    from ....ir.expr import Expr
    assert isinstance(node.get_operand(0).value, Expr)
    value = node.get_operand(0).value
    value = namify(value.as_operand())
    return f"sim.{value}_value.is_some()"


def _codegen_module_triggered(node, module_ctx):
    """Generate code for MODULE_TRIGGERED intrinsic."""
    port_self = dump_rval_ref(module_ctx, node.get_operand(0))
    return f"sim.{port_self}_triggered"


def _codegen_has_mem_resp(node, module_ctx):
    """Generate code for HAS_MEM_RESP intrinsic."""
    dram_module = node.args[0]
    dram_name = namify(dram_module.name)
    return f"sim.{dram_name}_response.valid"


def _codegen_get_mem_resp(node, module_ctx):
    """Generate code for GET_MEM_RESP intrinsic."""
    dram_module = node.args[0]
    dram_name = namify(dram_module.name)
    return f"BigUint::from_bytes_le(&sim.{dram_name}_response.data)"


def _codegen_external_output_read(node, module_ctx, **_kwargs):
    """Generate code for EXTERNAL_OUTPUT_READ intrinsic.

    This handles both WireOut (no index) and RegOut (with index) reads.
    Uses Verilator FFI getter methods (get_<port>()).
    Converts u8 to bool for Bits(1) types.
    """
    instance = node.args[0]  # ExternalIntrinsic
    port_name = node.args[1].value if hasattr(node.args[1], 'value') else node.args[1]

    # Optional: index parameter for RegOut (currently unused in codegen)
    # index = node.args[2] if len(node.args) > 2 else None

    instance_uid = instance.uid
    handle_name = f"external_{instance_uid}"

    # Get the port type to check if conversion is needed
    port_specs = instance.external_class.port_specs()
    wire_spec = port_specs.get(port_name)

    getter_call = f"sim.{handle_name}.get_{port_name}()"

    # Check if this is a Bits(1) port that needs u8 -> bool conversion
    if wire_spec and hasattr(wire_spec.dtype, 'bits') and wire_spec.dtype.bits == 1:
        # Verilator FFI returns u8, but simulator expects bool for Bits(1)
        return f"({getter_call} != 0)"

    return getter_call


# Dispatch table for pure intrinsic operations
_PURE_INTRINSIC_DISPATCH = {
    PureIntrinsic.FIFO_PEEK: _codegen_fifo_peek,
    PureIntrinsic.FIFO_VALID: _codegen_fifo_valid,
    PureIntrinsic.VALUE_VALID: _codegen_value_valid,
    PureIntrinsic.MODULE_TRIGGERED: _codegen_module_triggered,
    PureIntrinsic.HAS_MEM_RESP: _codegen_has_mem_resp,
    PureIntrinsic.GET_MEM_RESP: _codegen_get_mem_resp,
    PureIntrinsic.EXTERNAL_OUTPUT_READ: _codegen_external_output_read,
}


def codegen_pure_intrinsic(node: PureIntrinsic, module_ctx):
    """Generate code for pure intrinsic operations."""
    intrinsic = node.opcode
    if intrinsic == PureIntrinsic.CURRENT_CYCLE:
        # current_cycle returns cycle count in u64; stamp is usize time in half-cycles (50).
        # Divide by 100 to get cycles and cast to u64.
        return "((sim.stamp as u64) / 100u64)"
    codegen_func = _PURE_INTRINSIC_DISPATCH.get(intrinsic)
    if codegen_func is not None:
        return codegen_func(node, module_ctx)
    return None


def _codegen_wait_until(node, module_ctx):
    """Generate code for WAIT_UNTIL intrinsic."""
    value = dump_rval_ref(module_ctx, node.args[0])
    return f"if !{value} {{ return false; }}"


def _codegen_finish(node, module_ctx):
    """Generate code for FINISH intrinsic."""
    return "std::process::exit(0);"


def _codegen_assert(node, module_ctx):
    """Generate code for ASSERT intrinsic."""
    value = dump_rval_ref(module_ctx, node.args[0])
    return f"assert!({value});"


def _codegen_send_read_request(node, module_ctx):
    """Generate code for SEND_READ_REQUEST intrinsic."""
    dram_module = node.args[0]
    re = node.args[1]
    addr = node.args[2]
    dram_name = namify(dram_module.name)
    re_val = dump_rval_ref(module_ctx, re)
    addr_val = dump_rval_ref(module_ctx, addr)
    return f"""if {re_val} {{
                        unsafe {{
                            let mem_interface = &sim.mi_{dram_name};
                            let success = mem_interface.send_request(
                                {addr_val} as i64,
                                false,
                                crate::modules::{dram_name}::callback_of_{dram_name},
                                sim as *const _ as *mut _,
                            );
                            if success {{
                                sim.request_stamp_map_table.insert(
                                    {addr_val} as i64,
                                    sim.stamp,
                                );
                            }}
                            success
                        }}
                    }} else {{
                        false
                    }}"""


def _codegen_send_write_request(node, module_ctx):
    """Generate code for SEND_WRITE_REQUEST intrinsic."""
    dram_module = node.args[0]
    we = node.args[1]
    addr = node.args[2]
    data = node.args[3]  # pylint: disable=unused-variable
    dram_name = namify(dram_module.name)
    we_val = dump_rval_ref(module_ctx, we)
    addr_val = dump_rval_ref(module_ctx, addr)
    return f"""if {we_val} {{
                        unsafe {{
                            let mem_interface = &sim.mi_{dram_name};
                            let success = mem_interface.send_request(
                                {addr_val} as i64,
                                true,
                                crate::modules::{dram_name}::callback_of_{dram_name},
                                sim as *const _ as *mut _,
                            );
                            success
                        }}
                    }} else {{
                        false
                    }}"""


def _codegen_external_instantiate(node, module_ctx, **_kwargs):
    """Generate code for EXTERNAL_INSTANTIATE intrinsic.

    This handles the instantiation of external module instances.
    Uses Verilator FFI setter methods (set_<port>()).
    Converts bool to u8 for Bits(1) types.
    """
    # For ExternalIntrinsic, we need to assign input values and call eval()
    instance_uid = node.uid
    handle_name = f"external_{instance_uid}"

    # Get port specs to check types
    port_specs = node.external_class.port_specs()

    assignments = []
    for port_name, value in node.input_connections.items():
        value_code = dump_rval_ref(module_ctx, value)

        # Check if this is a Bits(1) port that needs bool -> u8 conversion
        wire_spec = port_specs.get(port_name)
        if wire_spec and hasattr(wire_spec.dtype, 'bits') and wire_spec.dtype.bits == 1:
            # Verilator FFI expects u8, but simulator uses bool for Bits(1)
            value_code = f"({value_code} as u8)"

        assignments.append(f"sim.{handle_name}.set_{port_name}({value_code});")

    # Call eval() to compute outputs from inputs
    assignments.append(f"sim.{handle_name}.eval();")

    if assignments:
        return "\n".join(assignments)
    return "/* External module instantiated */"


# Dispatch table for intrinsic operations
_INTRINSIC_DISPATCH = {
    Intrinsic.WAIT_UNTIL: _codegen_wait_until,
    Intrinsic.FINISH: _codegen_finish,
    Intrinsic.ASSERT: _codegen_assert,
    Intrinsic.SEND_READ_REQUEST: _codegen_send_read_request,
    Intrinsic.SEND_WRITE_REQUEST: _codegen_send_write_request,
    Intrinsic.EXTERNAL_INSTANTIATE: _codegen_external_instantiate,
    # PUSH/POP_CONDITION do not emit inline expressions here; handled at modules visitor
}


def codegen_intrinsic(node: Intrinsic, module_ctx):
    """Generate code for intrinsic operations."""
    # Handle ExternalIntrinsic specially
    if isinstance(node, ExternalIntrinsic):
        return _codegen_external_instantiate(node, module_ctx)

    intrinsic = node.opcode
    codegen_func = _INTRINSIC_DISPATCH.get(intrinsic)
    if codegen_func is not None:
        return codegen_func(node, module_ctx)
    return None
