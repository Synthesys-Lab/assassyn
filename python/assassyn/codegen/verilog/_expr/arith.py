# pylint: disable=too-many-branches
"""Arithmetic and logical operations code generation for Verilog.

This module contains functions to generate Verilog code for arithmetic operations
like BinaryOp, UnaryOp, as well as bit manipulation operations like Slice, Concat,
Cast, Select, and Select1Hot.
"""

from typing import Optional

from ....ir.expr import BinaryOp, UnaryOp, Concat, Cast, Select, Select1Hot
from ....ir.array import Slice
from ....ir.dtype import Bits
from ..utils import dump_type_cast, ensure_bits


def codegen_binary_op(dumper, expr: BinaryOp) -> Optional[str]:
    """Generate code for binary operations."""
    binop = expr.opcode
    dtype = expr.dtype

    lhs_type = expr.lhs.dtype
    rhs_type = expr.rhs.dtype

    a = dumper.dump_rval(expr.lhs, False)
    b = dumper.dump_rval(expr.rhs, False)
    rval = dumper.dump_rval(expr, False)

    if binop in [BinaryOp.SHL, BinaryOp.SHR] or 'SHR' in str(binop):
        if lhs_type.bits != rhs_type.bits:
            b = \
            f"BitsSignal.concat([Bits({lhs_type.bits - rhs_type.bits})(0), {b}.as_bits()])"

        b = f"{b}.as_bits()"
        a = f"{a}.as_bits()"

        op_class_name = None
        if binop == BinaryOp.SHL:
            op_class_name = "comb.ShlOp"
        elif binop == BinaryOp.SHR:
            if expr.lhs.dtype.is_signed():
                op_class_name = "comb.ShrSOp"
            else:
                op_class_name = "comb.ShrUOp"

        if op_class_name is None:
            raise TypeError(f"Unhandled shift operation: {binop}")
        return (
            f"{rval} = {op_class_name}({a}.as_bits(), {b}.as_bits())"
            f".as_bits({dtype.bits})[0:{dtype.bits}]"
            f".{dump_type_cast(dtype)}"
        )

    if binop == BinaryOp.MOD:
        if expr.dtype.is_signed():
            op_class_name = "comb.ModSOp"
        else:
            op_class_name = "comb.ModUOp"
        return (
            f"{rval} = {op_class_name}({a}.as_bits(), {b}.as_bits())"
            f".as_bits({dtype.bits})[0:{dtype.bits}]"
            f".{dump_type_cast(dtype)}"
        )

    if expr.is_comparative():
        # Convert to uint for comparison
        if not expr.lhs.dtype.is_int():
            a = f"{a}.as_uint()"
        if not expr.rhs.dtype.is_int():
            b = f"{b}.as_uint()"
        op_str = BinaryOp.OPERATORS[expr.opcode]
        op_body = f"(({a} {op_str} {b}).{dump_type_cast(dtype)})"
        return f'{rval} = {op_body}'

    # Default case for other binary operations
    op_str = BinaryOp.OPERATORS[expr.opcode]
    if expr.lhs.dtype != expr.rhs.dtype:
        b = f"{b}.{dump_type_cast(expr.lhs.dtype)}"
    if op_str == "&":
        if expr.rhs.dtype != Bits:
            b = f"{b}.as_bits()"
    op_body = f"(({a} {op_str} {b}).{dump_type_cast(dtype)})"
    return f'{rval} = {op_body}'


def codegen_unary_op(dumper, expr: UnaryOp) -> Optional[str]:
    """Generate code for unary operations."""
    uop = expr.opcode
    target_cast_str = dump_type_cast(expr.dtype)
    op_str = "~" if uop == UnaryOp.FLIP else "-"
    x = dumper.dump_rval(expr.x, False)
    rval = dumper.dump_rval(expr, False)
    if uop == UnaryOp.FLIP:
        x = f"({x}.as_bits())"
    body = f"{op_str}{x}"
    return f'{rval} = ({body}).{target_cast_str}'


def codegen_slice(dumper, expr: Slice) -> Optional[str]:
    """Generate code for slice operations."""
    a = dumper.dump_rval(expr.x, False)
    l = expr.l.value.value
    r = expr.r.value.value
    rval = dumper.dump_rval(expr, False)
    return f"{rval} = {a}.as_bits()[{l}:{r+1}]"


def codegen_concat(dumper, expr: Concat) -> Optional[str]:
    """Generate code for concatenation operations."""
    a = dumper.dump_rval(expr.msb, False)
    b = dumper.dump_rval(expr.lsb, False)
    rval = dumper.dump_rval(expr, False)
    return f"{rval} = BitsSignal.concat([{a}.as_bits(), {b}.as_bits()])"


def codegen_cast(dumper, expr: Cast) -> Optional[str]:
    """Generate code for cast operations."""
    dbits = expr.dtype.bits
    a = dumper.dump_rval(expr.x, False)
    src_dtype = expr.x.dtype
    pad = dbits - src_dtype.bits
    cast_body = ""
    cast_kind = expr.opcode
    rval = dumper.dump_rval(expr, False)

    if cast_kind == Cast.BITCAST:
        cast_body = f"{a}.{dump_type_cast(expr.dtype, dbits)}"
    elif cast_kind == Cast.ZEXT:
        cast_body = (
            f" BitsSignal.concat( [Bits({pad})(0) , {a}.as_bits()])"
            f".{dump_type_cast(expr.dtype)} "
        )
    elif cast_kind == Cast.SEXT:
        cast_body = (
            f"BitsSignal.concat( [BitsSignal.concat([ {a}.as_bits()[{src_dtype.bits-1}] ]"
            f" * {pad}) , {a}.as_bits()]).{dump_type_cast(expr.dtype)}"
        )
    return f"{rval} = {cast_body}"


def codegen_select(dumper, expr: Select) -> Optional[str]:
    """Generate code for select operations."""
    cond = dumper.dump_rval(expr.cond, False)
    true_value = dumper.dump_rval(expr.true_value, False)
    false_value = dumper.dump_rval(expr.false_value, False)
    rval = dumper.dump_rval(expr, False)

    cond = ensure_bits(cond)

    if expr.true_value.dtype != expr.false_value.dtype:
        false_value = f"{false_value}.{dump_type_cast(expr.true_value)}"
    return f'{rval} = Mux({cond}, {false_value}, {true_value})'


def codegen_select1hot(dumper, expr: Select1Hot) -> Optional[str]:
    """Generate code for 1-hot select operations."""
    rval = dumper.dump_rval(expr, False)
    cond = dumper.dump_rval(expr.cond, False)
    values = [dumper.dump_rval(v, False) for v in expr.values]

    if len(values) == 1:
        return f"{rval} = {values[0]}"

    num_values = len(values)
    selector_bits = max((num_values - 1).bit_length(), 1)
    if num_values == 2:
        body = f"{cond}.as_bits()[1]"
    else:
        dumper.append_code(f"{cond}_res = Bits({selector_bits})(0)")
        for i in range(num_values):
            dumper.append_code(
                f"{cond}_res = Mux({cond}[{i}] ,"
                f" {cond}_res , Bits({selector_bits})({i}))")

        values_str = ", ".join(values)
        mux_code = f"{rval} = Mux({cond}_res, {values_str})"
        dumper.append_code(mux_code)
        return None

    return body
