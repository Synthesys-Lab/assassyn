"""Expression code generation helpers for simulator.

This module contains helper functions to generate simulator code for different expression types.
"""

# pylint: disable=too-many-return-statements, too-many-branches, too-many-statements
# pylint: disable=unused-argument, too-many-locals, import-outside-toplevel

from ...ir.expr import (
    BinaryOp,
    UnaryOp,
    ArrayRead,
    ArrayWrite,
    Cast,
    AsyncCall,
    FIFOPop,
    FIFOPush,
    Log,
    Select,
    Select1Hot,
    Concat,
)
from ...ir.expr.intrinsic import PureIntrinsic, Intrinsic
from ...ir.expr.call import Bind
from ...ir.array import Slice
from .utils import dtype_to_rust_type, fifo_name
from ...utils import namify
from .node_dumper import dump_rval_ref
from .array import codegen_array_read, codegen_array_write


def codegen_binary_op(node: BinaryOp, module_ctx, sys):
    """Generate code for binary operations."""
    binop = BinaryOp.OPERATORS[node.opcode]

    if node.is_comparative():
        rust_ty = node.lhs.dtype
    else:
        rust_ty = node.dtype

    rust_ty = dtype_to_rust_type(rust_ty)
    lhs = dump_rval_ref(module_ctx, sys, node.lhs)
    rhs = dump_rval_ref(module_ctx, sys, node.rhs)

    # Special handling for shift operations with signed values
    if node.opcode == BinaryOp.SHR and node.lhs.dtype.is_signed():
        # For signed right shift, cast to signed type first
        if node.lhs.dtype.bits <= 64:
            lhs = f"ValueCastTo::<i{node.lhs.dtype.bits}>::cast(&{lhs})"
            rhs = f"ValueCastTo::<i{node.lhs.dtype.bits}>::cast(&{rhs})"
        else:
            lhs = f"ValueCastTo::<BigInt>::cast(&{lhs})"
            rhs = f"ValueCastTo::<BigInt>::cast(&{rhs})"
    else:
        lhs = f"ValueCastTo::<{rust_ty}>::cast(&{lhs})"
        rhs = f"ValueCastTo::<{rust_ty}>::cast(&{rhs})"

    return f"{lhs} {binop} {rhs}"


def codegen_unary_op(node: UnaryOp, module_ctx, sys):
    """Generate code for unary operations."""
    operand = dump_rval_ref(module_ctx, sys, node.x)
    uniop = UnaryOp.OPERATORS[node.opcode]
    return f"{uniop}{operand}"


def codegen_async_call(node: AsyncCall, module_ctx, sys):
    """Generate code for async call operations."""
    bind = node.bind
    event_q = f"{namify(bind.callee.name)}_event"
    return f"""{{
              let stamp = sim.stamp - sim.stamp % 100 + 100;
              sim.{event_q}.push_back(stamp)
            }}"""


def codegen_fifo_pop(node: FIFOPop, module_ctx, sys, module_name):
    """Generate code for FIFO pop operations."""
    fifo = node.fifo
    fifo_id = fifo_name(fifo)

    return f"""{{
              let stamp = sim.stamp - sim.stamp % 100 + 50;
              sim.{fifo_id}.pop.push(FIFOPop::new(stamp, "{module_name}"));
              match sim.{fifo_id}.payload.front() {{
                Some(value) => value.clone(),
                None => return false,
              }}
            }}"""


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
        from ...ir.expr import Expr
        assert isinstance(node.get_operand(0).value, Expr)
        value = node.get_operand(0).value
        value = namify(value.as_operand())
        return f"sim.{value}_value.is_some()"

    if intrinsic == PureIntrinsic.MODULE_TRIGGERED:
        port_self = dump_rval_ref(module_ctx, sys, node.get_operand(0))
        return f"sim.{port_self}_triggered"

    return None


def codegen_fifo_push(node: FIFOPush, module_ctx, sys, module_name):
    """Generate code for FIFO push operations."""
    fifo = node.fifo
    fifo_id = fifo_name(fifo)
    value = dump_rval_ref(module_ctx, sys, node.val)

    return f"""{{
              let stamp = sim.stamp;
              sim.{fifo_id}.push.push(
                FIFOPush::new(stamp + 50, {value}.clone(), "{module_name}"));
            }}"""


def codegen_log(node: Log, module_ctx, sys, module_name):
    """Generate code for log operations."""
    result = [f'print!("@line:{{:<5}} {{:<10}}: [{module_name}]\\t", line!(), cyclize(sim.stamp));']
    result.append("println!(")
    result.append(f"{dump_rval_ref(module_ctx, sys, node.operands[0])}, ")

    for elem in node.operands[1:]:
        dump = dump_rval_ref(module_ctx, sys, elem)
        dtype = elem.dtype
        if dtype.bits == 1:
            dump = f"if {dump} {{ 1 }} else {{ 0 }}"
        result.append(f"{dump}, ")

    result.append(")")
    return "".join(result)


def codegen_slice(node: Slice, module_ctx, sys):
    """Generate code for slice operations."""
    a = dump_rval_ref(module_ctx, sys, node.x)
    l = node.l.value.value
    r = node.r.value.value
    dtype = node.dtype
    num_bits = r - l + 1
    mask_bits = "1" * num_bits

    if l < 64 and r < 64:
        result_a = f'''let a = ValueCastTo::<u64>::cast(&{a});
                               let mask = u64::from_str_radix("{mask_bits}", 2).unwrap();'''
    else:
        result_a = f'''let a = ValueCastTo::<BigUint>::cast(&{a});
let mask = BigUint::parse_bytes("{mask_bits}".as_bytes(), 2).unwrap();'''

    return f"""{{
                {result_a}
                let res = (a >> {l}) & mask;
                ValueCastTo::<{dtype_to_rust_type(dtype)}>::cast(&res)
            }}"""


def codegen_concat(node: Concat, module_ctx, sys):
    """Generate code for concatenation operations."""
    dtype = node.dtype
    a = dump_rval_ref(module_ctx, sys, node.msb)
    b = dump_rval_ref(module_ctx, sys, node.lsb)
    b_bits = node.lsb.dtype.bits

    return f"""{{
                let a = ValueCastTo::<BigUint>::cast(&{a});
                let b = ValueCastTo::<BigUint>::cast(&{b});
                let c = (a << {b_bits}) | b;
                ValueCastTo::<{dtype_to_rust_type(dtype)}>::cast(&c)
            }}"""


def codegen_select(node: Select, module_ctx, sys):
    """Generate code for select operations."""
    cond = dump_rval_ref(module_ctx, sys, node.cond)
    true_value = dump_rval_ref(module_ctx, sys, node.true_value)
    false_value = dump_rval_ref(module_ctx, sys, node.false_value)
    return f"if {cond} {{ {true_value} }} else {{ {false_value} }}"


def codegen_select1hot(node: Select1Hot, module_ctx, sys):
    """Generate code for 1-hot select operations."""
    cond = dump_rval_ref(module_ctx, sys, node.cond)
    target_type = dtype_to_rust_type(node.dtype)
    result = [f'''{{ let cond = {cond};
assert!(cond.count_ones() == 1, "Select1Hot: condition is not 1-hot");''']

    for i, value in enumerate(node.values):
        if i != 0:
            result.append(" else ")
        value_ref = dump_rval_ref(module_ctx, sys, value)
        result.append(f'''if cond >> {i} & 1 != 0
{{ ValueCastTo::<{target_type}>::cast(&{value_ref}) }}''')

    result.append(" else { unreachable!() } }")
    return "".join(result)


def codegen_cast(node: Cast, module_ctx, sys):
    """Generate code for cast operations."""
    dest_dtype = node.dtype
    a = dump_rval_ref(module_ctx, sys, node.x)

    if node.opcode in [Cast.ZEXT, Cast.BITCAST, Cast.SEXT]:
        return f"ValueCastTo::<{dtype_to_rust_type(dest_dtype)}>::cast(&{a})"

    return None


def codegen_bind(node: Bind, module_ctx, sys):
    """Generate code for bind operations."""
    return "()"


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


def codegen_expr(node, module_ctx, sys, module_name, modules_for_callback):
    """Generate code for an expression node.

    This is the main dispatcher function that delegates to specific codegen functions
    based on the expression type.
    """
    if isinstance(node, BinaryOp):
        return codegen_binary_op(node, module_ctx, sys)

    if isinstance(node, UnaryOp):
        return codegen_unary_op(node, module_ctx, sys)

    if isinstance(node, ArrayRead):
        return codegen_array_read(node, module_ctx, sys)

    if isinstance(node, ArrayWrite):
        return codegen_array_write(node, module_ctx, sys, module_name)

    if isinstance(node, AsyncCall):
        return codegen_async_call(node, module_ctx, sys)

    if isinstance(node, FIFOPop):
        return codegen_fifo_pop(node, module_ctx, sys, module_name)

    if isinstance(node, PureIntrinsic):
        return codegen_pure_intrinsic(node, module_ctx, sys)

    if isinstance(node, FIFOPush):
        return codegen_fifo_push(node, module_ctx, sys, module_name)

    if isinstance(node, Log):
        return codegen_log(node, module_ctx, sys, module_name)

    if isinstance(node, Slice):
        return codegen_slice(node, module_ctx, sys)

    if isinstance(node, Concat):
        return codegen_concat(node, module_ctx, sys)

    if isinstance(node, Select):
        return codegen_select(node, module_ctx, sys)

    if isinstance(node, Select1Hot):
        return codegen_select1hot(node, module_ctx, sys)

    if isinstance(node, Cast):
        return codegen_cast(node, module_ctx, sys)

    if isinstance(node, Bind):
        return codegen_bind(node, module_ctx, sys)

    if isinstance(node, Intrinsic):
        return codegen_intrinsic(node, module_ctx, sys, module_name, modules_for_callback)

    return None
