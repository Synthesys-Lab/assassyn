# pylint: disable=too-many-return-statements,too-many-branches
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
    if isinstance(expr, BinaryOp):
        return codegen_binary_op(dumper, expr)

    if isinstance(expr, UnaryOp):
        return codegen_unary_op(dumper, expr)

    if isinstance(expr, Log):
        return codegen_log(dumper, expr)

    if isinstance(expr, ArrayRead):
        return codegen_array_read(dumper, expr)

    if isinstance(expr, ArrayWrite):
        return codegen_array_write(dumper, expr)

    if isinstance(expr, FIFOPush):
        return codegen_fifo_push(dumper, expr)

    if isinstance(expr, FIFOPop):
        return codegen_fifo_pop(dumper, expr)

    if isinstance(expr, PureIntrinsic):
        return codegen_pure_intrinsic(dumper, expr)

    if isinstance(expr, AsyncCall):
        return codegen_async_call(dumper, expr)

    if isinstance(expr, Slice):
        return codegen_slice(dumper, expr)

    if isinstance(expr, Concat):
        return codegen_concat(dumper, expr)

    if isinstance(expr, Cast):
        return codegen_cast(dumper, expr)

    if isinstance(expr, Select):
        return codegen_select(dumper, expr)

    if isinstance(expr, Bind):
        return codegen_bind(dumper, expr)

    if isinstance(expr, Select1Hot):
        return codegen_select1hot(dumper, expr)

    if isinstance(expr, Intrinsic):
        return codegen_intrinsic(dumper, expr)

    if isinstance(expr, WireAssign):
        return codegen_wire_assign(dumper, expr)

    if isinstance(expr, WireRead):
        return codegen_wire_read(dumper, expr)

    raise ValueError(f"Unhandled expression type: {type(expr).__name__}")
