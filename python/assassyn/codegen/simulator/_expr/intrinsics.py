"""Intrinsic code generation helpers for simulator.

This module contains helper functions to generate simulator code for intrinsic operations.
"""

# pylint: disable=too-many-return-statements, too-many-locals, unused-argument
# pylint: disable=import-outside-toplevel

from ....ir.expr.intrinsic import PureIntrinsic, Intrinsic
from ..utils import fifo_name
from ....utils import namify
from ..node_dumper import dump_rval_ref


def codegen_pure_intrinsic(node: PureIntrinsic, module_ctx, sys):
    """Generate code for pure intrinsic operations."""
    intrinsic = node.opcode

    if intrinsic == PureIntrinsic.FIFO_PEEK:
        port_self = dump_rval_ref(module_ctx, sys, node.get_operand(0))
        return f"sim.{port_self}.front().cloned()"

    if intrinsic == PureIntrinsic.FIFO_VALID:
        port_self = dump_rval_ref(module_ctx, sys, node.get_operand(0))
        return f"!sim.{port_self}.is_empty()"

    if intrinsic == PureIntrinsic.VALUE_VALID:
        from ....ir.expr import Expr
        assert isinstance(node.get_operand(0).value, Expr)
        value = node.get_operand(0).value
        value = namify(value.as_operand())
        return f"sim.{value}_value.is_some()"

    if intrinsic == PureIntrinsic.MODULE_TRIGGERED:
        port_self = dump_rval_ref(module_ctx, sys, node.get_operand(0))
        return f"sim.{port_self}_triggered"

    return None


def codegen_intrinsic(node: Intrinsic, module_ctx, sys, module_name, modules_for_callback):
    """Generate code for intrinsic operations."""
    intrinsic = node.opcode

    if intrinsic == Intrinsic.WAIT_UNTIL:
        value = dump_rval_ref(module_ctx, sys, node.args[0])
        return f"if !{value} {{ return false; }}"

    if intrinsic == Intrinsic.FINISH:
        return "std::process::exit(0);"

    if intrinsic == Intrinsic.ASSERT:
        value = dump_rval_ref(module_ctx, sys, node.args[0])
        return f"assert!({value});"

    if intrinsic == Intrinsic.BARRIER:
        return "/* Barrier */"

    if intrinsic == Intrinsic.SEND_READ_REQUEST:
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

    if intrinsic == Intrinsic.SEND_WRITE_REQUEST:
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

    if intrinsic == Intrinsic.USE_DRAM:
        fifo = node.args[0]
        fifo_id = fifo_name(fifo)
        modules_for_callback["MemUser_rdata"] = fifo_id
        return None

    if intrinsic == Intrinsic.HAS_MEM_RESP:
        val = dump_rval_ref(module_ctx, sys, node)
        if not modules_for_callback.get("MemUser_rdata"):
            return f"let {val} = false"
        mem_rdata = modules_for_callback["MemUser_rdata"]
        return f"let {val} = sim.{mem_rdata}.payload.is_empty() == false"

    if intrinsic == Intrinsic.MEM_RESP:
        val = dump_rval_ref(module_ctx, sys, node)
        if not modules_for_callback.get("MemUser_rdata"):
            return f"let {val} = 0"
        mem_rdata = modules_for_callback["MemUser_rdata"]
        return f"let {val} = sim.{mem_rdata}.payload.front().unwrap().clone()"

    if intrinsic == Intrinsic.MEM_WRITE:
        array = node.args[0]
        idx = node.args[1]
        value = node.args[2]
        array_name = namify(array.name)
        idx_val = dump_rval_ref(module_ctx, sys, idx)
        value_val = dump_rval_ref(module_ctx, sys, value)
        modules_for_callback["memory"] = module_name
        modules_for_callback["store"] = array_name
        port_id = id("DRAM")
        return f"""{{
                    let stamp = sim.stamp - sim.stamp % 100 + 50;
                    sim.{array_name}.write_port.push(
                        ArrayWrite::new(stamp, {idx_val} as usize, {value_val}.clone(), "{module_name}", {port_id}));
                }}"""

    return None
