"""Intrinsic code generation helpers for simulator.

This module contains helper functions to generate simulator code for intrinsic operations.
"""

# pylint: disable=too-many-locals, unused-argument
# pylint: disable=import-outside-toplevel

from ....ir.expr.intrinsic import PureIntrinsic, Intrinsic
from ....utils import namify
from ..node_dumper import dump_rval_ref


def _codegen_fifo_peek(node, module_ctx, sys, **_kwargs):
    """Generate code for FIFO_PEEK intrinsic."""
    port_self = dump_rval_ref(module_ctx, sys, node.get_operand(0))
    return f"sim.{port_self}.front().cloned()"


def _codegen_fifo_valid(node, module_ctx, sys, **_kwargs):
    """Generate code for FIFO_VALID intrinsic."""
    port_self = dump_rval_ref(module_ctx, sys, node.get_operand(0))
    return f"!sim.{port_self}.is_empty()"


def _codegen_value_valid(node, module_ctx, sys, **_kwargs):
    """Generate code for VALUE_VALID intrinsic."""
    from ....ir.expr import Expr
    assert isinstance(node.get_operand(0).value, Expr)
    value = node.get_operand(0).value
    value = namify(value.as_operand())
    return f"sim.{value}_value.is_some()"


def _codegen_module_triggered(node, module_ctx, sys, **_kwargs):
    """Generate code for MODULE_TRIGGERED intrinsic."""
    port_self = dump_rval_ref(module_ctx, sys, node.get_operand(0))
    return f"sim.{port_self}_triggered"


def _codegen_has_mem_resp(node, module_ctx, sys, **_kwargs):
    """Generate code for HAS_MEM_RESP intrinsic."""
    dram_module = node.args[0]
    dram_name = namify(dram_module.name)
    return f"sim.{dram_name}_response.valid"


def _codegen_get_mem_resp(node, module_ctx, sys, **_kwargs):
    """Generate code for GET_MEM_RESP intrinsic."""
    dram_module = node.args[0]
    dram_name = namify(dram_module.name)
    # Convert Vec<u8> to BigUint using from_bytes_le as documented
    return f"BigUint::from_bytes_le(&sim.{dram_name}_response.data)"


# Dispatch table for pure intrinsic operations
_PURE_INTRINSIC_DISPATCH = {
    PureIntrinsic.FIFO_PEEK: _codegen_fifo_peek,
    PureIntrinsic.FIFO_VALID: _codegen_fifo_valid,
    PureIntrinsic.VALUE_VALID: _codegen_value_valid,
    PureIntrinsic.MODULE_TRIGGERED: _codegen_module_triggered,
    PureIntrinsic.HAS_MEM_RESP: _codegen_has_mem_resp,
    PureIntrinsic.GET_MEM_RESP: _codegen_get_mem_resp,
}


def codegen_pure_intrinsic(node: PureIntrinsic, module_ctx, sys):
    """Generate code for pure intrinsic operations."""
    intrinsic = node.opcode
    codegen_func = _PURE_INTRINSIC_DISPATCH.get(intrinsic)
    if codegen_func is not None:
        return codegen_func(node, module_ctx, sys)
    return None


def _codegen_wait_until(node, module_ctx, sys, **_kwargs):
    """Generate code for WAIT_UNTIL intrinsic."""
    value = dump_rval_ref(module_ctx, sys, node.args[0])
    return f"if !{value} {{ return false; }}"


def _codegen_finish(node, module_ctx, sys, **_kwargs):
    """Generate code for FINISH intrinsic."""
    return "std::process::exit(0);"


def _codegen_assert(node, module_ctx, sys, **_kwargs):
    """Generate code for ASSERT intrinsic."""
    value = dump_rval_ref(module_ctx, sys, node.args[0])
    return f"assert!({value});"


def _codegen_barrier(node, module_ctx, sys, **_kwargs):
    """Generate code for BARRIER intrinsic."""
    return "/* Barrier */"


def _codegen_send_read_request(node, module_ctx, sys, **_kwargs):
    """Generate code for SEND_READ_REQUEST intrinsic."""
    dram_module = node.args[0]
    re = node.args[1]
    addr = node.args[2]
    dram_name = namify(dram_module.name)
    re_val = dump_rval_ref(module_ctx, sys, re)
    addr_val = dump_rval_ref(module_ctx, sys, addr)
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


def _codegen_send_write_request(node, module_ctx, sys, **_kwargs):
    """Generate code for SEND_WRITE_REQUEST intrinsic."""
    dram_module = node.args[0]
    we = node.args[1]
    addr = node.args[2]
    data = node.args[3]  # pylint: disable=unused-variable
    dram_name = namify(dram_module.name)
    we_val = dump_rval_ref(module_ctx, sys, we)
    addr_val = dump_rval_ref(module_ctx, sys, addr)
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






# Dispatch table for intrinsic operations
_INTRINSIC_DISPATCH = {
    Intrinsic.WAIT_UNTIL: _codegen_wait_until,
    Intrinsic.FINISH: _codegen_finish,
    Intrinsic.ASSERT: _codegen_assert,
    Intrinsic.BARRIER: _codegen_barrier,
    Intrinsic.SEND_READ_REQUEST: _codegen_send_read_request,
    Intrinsic.SEND_WRITE_REQUEST: _codegen_send_write_request,
}


def codegen_intrinsic(node: Intrinsic, module_ctx, sys, **kwargs):
    """Generate code for intrinsic operations."""
    intrinsic = node.opcode
    codegen_func = _INTRINSIC_DISPATCH.get(intrinsic)
    if codegen_func is not None:
        return codegen_func(
            node,
            module_ctx,
            sys,
            **kwargs
        )
    return None
