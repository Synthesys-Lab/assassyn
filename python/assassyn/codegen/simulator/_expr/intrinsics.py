"""Intrinsic code generation helpers for simulator.

This module contains helper functions to generate simulator code for intrinsic operations.
"""

# pylint: disable=too-many-locals, unused-argument
# pylint: disable=import-outside-toplevel

from ....ir.expr.intrinsic import PureIntrinsic, Intrinsic
from ....utils import namify
from ..callback_collector import get_current_callback_metadata
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


# Dispatch table for pure intrinsic operations
_PURE_INTRINSIC_DISPATCH = {
    PureIntrinsic.FIFO_PEEK: _codegen_fifo_peek,
    PureIntrinsic.FIFO_VALID: _codegen_fifo_valid,
    PureIntrinsic.VALUE_VALID: _codegen_value_valid,
    PureIntrinsic.MODULE_TRIGGERED: _codegen_module_triggered,
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
    idx = node.args[0]
    idx_val = dump_rval_ref(module_ctx, sys, idx)
    return f"""{{
                    unsafe {{
                        let mem_interface = &sim.mem_interface;
                        let success = mem_interface.send_request({idx_val} as i64, false, rust_callback, sim as *const _ as *mut _,);
                        if success {{
                            sim.request_stamp_map_table.insert({idx_val} as i64, sim.stamp);
                        }}
                        success
                    }}
                }}"""


def _codegen_send_write_request(node, module_ctx, sys, **_kwargs):
    """Generate code for SEND_WRITE_REQUEST intrinsic."""
    idx = node.args[0]
    we = node.args[1]
    idx_val = dump_rval_ref(module_ctx, sys, idx)
    we_val = dump_rval_ref(module_ctx, sys, we)
    val = dump_rval_ref(module_ctx, sys, node)
    return f"""
                    let {val} = unsafe {{
                        if {we_val} {{
                            let mem_interface = &sim.mem_interface;
                            let success = mem_interface.send_request({idx_val} as i64, true, rust_callback, sim as *const _ as *mut _,);
                            success
                        }} else {{
                            false
                        }}
                    }};
                """


def _codegen_use_dram(node, module_ctx, sys, **_kwargs):
    """Generate code for USE_DRAM intrinsic (metadata handled elsewhere)."""
    return None


def _codegen_has_mem_resp(node, module_ctx, sys, **_kwargs):
    """Generate code for HAS_MEM_RESP intrinsic."""
    metadata = get_current_callback_metadata()
    val = dump_rval_ref(module_ctx, sys, node)
    mem_rdata = metadata.mem_user_rdata
    if not mem_rdata:
        return f"let {val} = false"
    return f"let {val} = sim.{mem_rdata}.payload.is_empty() == false"


def _codegen_mem_resp(node, module_ctx, sys, **_kwargs):
    """Generate code for MEM_RESP intrinsic."""
    metadata = get_current_callback_metadata()
    val = dump_rval_ref(module_ctx, sys, node)
    mem_rdata = metadata.mem_user_rdata
    if not mem_rdata:
        return f"let {val} = 0"
    return f"let {val} = sim.{mem_rdata}.payload.front().unwrap().clone()"


def _codegen_mem_write(node, module_ctx, sys):
    """Generate code for MEM_WRITE intrinsic."""
    module_name = module_ctx.name
    array = node.args[0]
    idx = node.args[1]
    value = node.args[2]
    array_name = namify(array.name)
    idx_val = dump_rval_ref(module_ctx, sys, idx)
    value_val = dump_rval_ref(module_ctx, sys, value)
    port_id = id("DRAM")
    return f"""{{
                    let stamp = sim.stamp - sim.stamp % 100 + 50;
                    sim.{array_name}.write_port.push(
                        ArrayWrite::new(stamp, {idx_val} as usize, {value_val}.clone(), "{module_name}", {port_id}));
                }}"""


# Dispatch table for intrinsic operations
_INTRINSIC_DISPATCH = {
    Intrinsic.WAIT_UNTIL: _codegen_wait_until,
    Intrinsic.FINISH: _codegen_finish,
    Intrinsic.ASSERT: _codegen_assert,
    Intrinsic.BARRIER: _codegen_barrier,
    Intrinsic.SEND_READ_REQUEST: _codegen_send_read_request,
    Intrinsic.SEND_WRITE_REQUEST: _codegen_send_write_request,
    Intrinsic.USE_DRAM: _codegen_use_dram,
    Intrinsic.HAS_MEM_RESP: _codegen_has_mem_resp,
    Intrinsic.MEM_RESP: _codegen_mem_resp,
    Intrinsic.MEM_WRITE: _codegen_mem_write,
}


def codegen_intrinsic(node: Intrinsic, module_ctx, sys):
    """Generate code for intrinsic operations."""
    intrinsic = node.opcode
    codegen_func = _INTRINSIC_DISPATCH.get(intrinsic)
    if codegen_func is not None:
        return codegen_func(
            node,
            module_ctx,
            sys,
        )
    return None
