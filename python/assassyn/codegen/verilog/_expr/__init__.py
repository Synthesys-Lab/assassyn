"""Verilog expression code generation.

This module contains functions to generate Verilog code for different expression types.
"""

from typing import Optional

from ....ir.expr import (
    BinaryOp,
    UnaryOp,
    Log,
    ArrayRead,
    ArrayWrite,
    FIFOPop,
    FIFOPush,
    AsyncCall,
    Concat,
    Cast,
    Select,
    Select1Hot,
    WireAssign,
    WireRead,
)
from ....ir.expr.intrinsic import PureIntrinsic, Intrinsic
from ....ir.expr.call import Bind
from ....ir.array import Slice

from .arith import (
    codegen_binary_op,
    codegen_unary_op,
    codegen_slice,
    codegen_concat,
    codegen_cast,
    codegen_select,
    codegen_select1hot,
)
from .array import (
    codegen_array_read,
    codegen_array_write,
    codegen_fifo_push,
    codegen_fifo_pop,
)
from .call import (
    codegen_async_call,
    codegen_bind,
    codegen_wire_assign,
    codegen_wire_read,
)
from .intrinsics import (
    codegen_pure_intrinsic,
    codegen_intrinsic,
    codegen_log,
)

# Dispatch table mapping expression types to their codegen functions
_EXPR_CODEGEN_DISPATCH = {
    BinaryOp: codegen_binary_op,
    UnaryOp: codegen_unary_op,
    Log: codegen_log,
    ArrayRead: codegen_array_read,
    ArrayWrite: codegen_array_write,
    FIFOPush: codegen_fifo_push,
    FIFOPop: codegen_fifo_pop,
    PureIntrinsic: codegen_pure_intrinsic,
    AsyncCall: codegen_async_call,
    Slice: codegen_slice,
    Concat: codegen_concat,
    Cast: codegen_cast,
    Select: codegen_select,
    Bind: codegen_bind,
    Select1Hot: codegen_select1hot,
    Intrinsic: codegen_intrinsic,
    WireAssign: codegen_wire_assign,
    WireRead: codegen_wire_read,
}


def codegen_expr(dumper, expr) -> Optional[str]:
    """Generate code for an expression node.

    This is the main dispatcher function that delegates to specific codegen functions
    based on the expression type.

    Args:
        dumper: The CIRCTDumper instance
        expr: The expression node to generate code for

    Returns:
        Generated code string or None
    """
    expr_type = type(expr)

    # Try exact match first
    codegen_func = _EXPR_CODEGEN_DISPATCH.get(expr_type)
    if codegen_func is not None:
        return codegen_func(dumper, expr)

    # Fall back to isinstance check for subclasses
    for base_type, func in _EXPR_CODEGEN_DISPATCH.items():
        if isinstance(expr, base_type):
            return func(dumper, expr)

    raise ValueError(f"Unhandled expression type: {expr_type.__name__}")
