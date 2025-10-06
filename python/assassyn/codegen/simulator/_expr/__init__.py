"""Expression code generation helpers for simulator.

This module contains helper functions to generate simulator code for different expression types.
"""

# pylint: disable=unused-argument, too-many-locals, import-outside-toplevel

from ....ir.expr import (
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
from ....ir.expr.intrinsic import PureIntrinsic, Intrinsic
from ....ir.expr.call import Bind
from ....ir.array import Slice
from ..utils import dtype_to_rust_type
from ..node_dumper import dump_rval_ref
from .array import codegen_array_read, codegen_array_write
from .arith import codegen_binary_op, codegen_unary_op
from .intrinsics import codegen_intrinsic, codegen_pure_intrinsic
from .call import codegen_async_call, codegen_fifo_pop, codegen_fifo_push, codegen_bind


def codegen_log(node: Log, module_ctx, sys):
    """Generate code for log operations."""
    module_name = module_ctx.name
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

# Dispatch table mapping expression types to their codegen functions
_EXPR_CODEGEN_DISPATCH = {
    BinaryOp: codegen_binary_op,
    UnaryOp: codegen_unary_op,
    ArrayRead: codegen_array_read,
    ArrayWrite: codegen_array_write,
    AsyncCall: codegen_async_call,
    FIFOPop: codegen_fifo_pop,
    PureIntrinsic: codegen_pure_intrinsic,
    FIFOPush: codegen_fifo_push,
    Log: codegen_log,
    Slice: codegen_slice,
    Concat: codegen_concat,
    Select: codegen_select,
    Select1Hot: codegen_select1hot,
    Cast: codegen_cast,
    Bind: codegen_bind,
    Intrinsic: codegen_intrinsic,
}


def _call_codegen_func(func, node, module_ctx, sys, **kwargs):
    """Helper function to call codegen functions with appropriate parameters."""
    if func.__name__ == 'codegen_array_write':
        return func(node, module_ctx, sys, module_ctx.name)
    if func.__name__ == 'codegen_intrinsic':
        return func(node, module_ctx, sys, **kwargs)
    return func(node, module_ctx, sys)


def codegen_expr(node, module_ctx, sys, **kwargs):
    """Generate code for an expression node.

    This is the main dispatcher function that delegates to specific codegen functions
    based on the expression type.
    """
    node_type = type(node)

    # Try exact match first
    codegen_func = _EXPR_CODEGEN_DISPATCH.get(node_type)
    if codegen_func is not None:
        return _call_codegen_func(codegen_func, node, module_ctx, sys, **kwargs)

    # Fall back to isinstance check for subclasses
    for base_type, func in _EXPR_CODEGEN_DISPATCH.items():
        if isinstance(node, base_type):
            return _call_codegen_func(func, node, module_ctx, sys, **kwargs)

    return None
