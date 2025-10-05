"""Arithmetic operation code generation for simulator.

This module contains helper functions to generate simulator code for arithmetic operations.
"""

# pylint: disable=unused-argument

from ....ir.expr import BinaryOp, UnaryOp
from ..utils import dtype_to_rust_type
from ..node_dumper import dump_rval_ref


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
