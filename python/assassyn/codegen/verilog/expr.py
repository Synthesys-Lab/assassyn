# pylint: disable=too-many-lines,too-many-return-statements,too-many-branches
"""Verilog expression code generation.

This module contains functions to generate Verilog code for different expression types.
"""

from typing import Optional
from string import Formatter

from ...ir.expr import (
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
from ...ir.expr.intrinsic import PureIntrinsic, Intrinsic
from ...ir.expr.call import Bind
from ...ir.array import Slice
from ...ir.const import Const
from ...ir.dtype import Int, Record
from ...ir.block import CondBlock, CycledBlock
from ...ir.module.external import ExternalSV
from ...utils import unwrap_operand, namify
from .utils import dump_type, dump_type_cast, ensure_bits


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
        from ...ir.dtype import Bits
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


def codegen_log(dumper, expr: Log) -> Optional[str]:
    """Generate code for log operations."""
    formatter_str = expr.operands[0].value

    arg_print_snippets = []
    condition_snippets = []
    module_name = namify(dumper.current_module.name)

    for i in expr.operands[1:]:
        operand = unwrap_operand(i)
        if not isinstance(operand, Const):
            dumper.expose('expr', operand)
            exposed_name = dumper.dump_rval(operand, True)
            valid_signal = f'dut.{module_name}.valid_{exposed_name}.value'
            condition_snippets.append(valid_signal)

            base_value = f"dut.{module_name}.expose_{exposed_name}.value"
            if isinstance(operand.dtype, Int):
                bits = operand.dtype.bits
                expose_signal = (
                    f"({base_value} - (1 << {bits}) "
                    f"if ({base_value} >> ({bits} - 1)) & 1 else int({base_value}))"
                )
            else:
                expose_signal = f"int({base_value})"
            arg_print_snippets.append(expose_signal)
        else:
            arg_print_snippets.append(str(operand.value))

    f_string_content_parts = []
    arg_iterator = iter(arg_print_snippets)

    for literal_text, field_name, format_spec, conversion \
        in Formatter().parse(formatter_str):

        if literal_text:
            f_string_content_parts.append(literal_text)

        if field_name is not None:
            if format_spec == '?':
                conversion = 'r'
                format_spec = None
            arg_code = next(arg_iterator)
            new_placeholder = f"{{{arg_code}"
            if conversion:  # for !s, !r, !a
                new_placeholder += f"!{conversion}"
            if format_spec:  # for :b, :08x,
                new_placeholder += f":{format_spec}"
            new_placeholder += "}"
            f_string_content_parts.append(new_placeholder)

    f_string_content = "".join(f_string_content_parts)

    block_condition = dumper.get_pred()
    block_condition = block_condition.replace('cycle_count', 'dut.global_cycle_count')
    final_conditions = []

    for cond_str, cond_obj in dumper.cond_stack:
        if isinstance(cond_obj, CycledBlock):
            tb_cond_path = \
            cond_str.replace("self.cycle_count", "dut.global_cycle_count.value")
            final_conditions.append(tb_cond_path)

        elif isinstance(cond_obj, CondBlock):
            exposed_name = dumper.dump_rval(cond_obj.cond, True)

            tb_expose_path = f"(dut.{module_name}.expose_{exposed_name}.value)"
            tb_valid_path = f"(dut.{module_name}.valid_{exposed_name}.value)"

            combined_cond = f"({tb_valid_path} & {tb_expose_path})"
            final_conditions.append(combined_cond)

    if condition_snippets:
        final_conditions.append(" and ".join(condition_snippets))

    if_condition = " and ".join(final_conditions)

    dumper.logs.append(f'# {expr}')

    line_info = f"@line:{expr.loc.rsplit(':', 1)[-1]}"

    module_info = f"[{namify(dumper.current_module.name)}]"

    # pylint: disable-next=W1309
    cycle_info = f"Cycle @{{float(dut.global_cycle_count.value):.2f}}:"

    final_print_string = (
         f'f"{line_info} {cycle_info} {module_info:<20} {f_string_content}"'
     )

    dumper.logs.append(f'#@ line {expr.loc}: {expr}')
    if if_condition:
        dumper.logs.append(f'if ( {if_condition} ):')
        dumper.logs.append(f'    print({final_print_string})')
    else:
        dumper.logs.append(f'print({final_print_string})')

    return None


def codegen_array_read(dumper, expr: ArrayRead) -> Optional[str]:
    """Generate code for array read operations."""
    array_ref = expr.array
    is_sram_payload = False

    # Import SRAM here to avoid circular imports
    from ...ir.module import SRAM

    if isinstance(dumper.current_module, SRAM):
        if array_ref == dumper.current_module.payload:
            is_sram_payload = True

    rval = dumper.dump_rval(expr, False)

    if is_sram_payload:
        body = f'{rval} = self.mem_dataout'
        dumper.expose('array', expr)
    else:
        array_idx = unwrap_operand(expr.idx)
        array_idx = (dumper.dump_rval(array_idx, False)
                    if not isinstance(array_idx, Const) else array_idx.value)
        index_bits = array_ref.index_bits if array_ref.index_bits > 0 else 1
        from ...ir.dtype import Bits
        if dump_type(expr.idx.dtype) != Bits and not isinstance(array_idx, int):
            array_idx = f"{array_idx}.as_bits({index_bits})"

        array_name = dumper.dump_rval(array_ref, False)
        if isinstance(expr.dtype, Record):
            body = f'{rval} = self.{array_name}_q_in[{array_idx}]'
        else:
            body = \
            f'{rval} = self.{array_name}_q_in[{array_idx}].{dump_type_cast(expr.dtype)}'
        dumper.expose('array', expr)

    return body


def codegen_array_write(dumper, expr: ArrayWrite) -> Optional[str]:
    """Generate code for array write operations."""
    dumper.expose('array', expr)
    return None


def codegen_fifo_push(dumper, expr: FIFOPush) -> Optional[str]:
    """Generate code for FIFO push operations."""
    dumper.expose('fifo', expr)
    return None


def codegen_fifo_pop(dumper, expr: FIFOPop) -> Optional[str]:
    """Generate code for FIFO pop operations."""
    rval = namify(expr.as_operand())
    fifo_name = dumper.dump_rval(expr.fifo, False)
    dumper.expose('fifo_pop', expr)
    return f'{rval} = self.{fifo_name}'


def codegen_pure_intrinsic(dumper, expr: PureIntrinsic) -> Optional[str]:
    """Generate code for pure intrinsic operations."""
    intrinsic = expr.opcode
    rval = dumper.dump_rval(expr, False)

    if intrinsic in [PureIntrinsic.FIFO_VALID, PureIntrinsic.FIFO_PEEK]:
        fifo = expr.args[0]
        fifo_name = dumper.dump_rval(fifo, False)
        if intrinsic == PureIntrinsic.FIFO_PEEK:
            dumper.expose('expr', expr)
            return f'{rval} = self.{fifo_name}'
        if intrinsic == PureIntrinsic.FIFO_VALID:
            return f'{rval} = self.{fifo_name}_valid'
    elif intrinsic == PureIntrinsic.VALUE_VALID:
        value_expr = expr.operands[0].value
        if value_expr.parent.module != expr.parent.module:
            port_name = dumper.get_external_port_name(value_expr)
            return f"{rval} = self.{port_name}_valid"
        return f"{rval} = self.executed"
    else:
        raise ValueError(f"Unknown intrinsic: {expr}")

    return None


def codegen_async_call(dumper, expr: AsyncCall) -> Optional[str]:
    """Generate code for async call operations."""
    dumper.expose('trigger', expr)
    return None


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

    if expr.true_value.dtype != expr.false_value.dtype:
        false_value = f"{false_value}.{dump_type_cast(expr.true_value)}"
    return f'{rval} = Mux({cond}, {false_value}, {true_value})'


def codegen_bind(dumper, expr: Bind) -> Optional[str]:
    """Generate code for bind operations."""
    return None


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


def codegen_intrinsic(dumper, expr: Intrinsic) -> Optional[str]:
    """Generate code for intrinsic operations."""
    intrinsic = expr.opcode

    if intrinsic == Intrinsic.FINISH:
        predicate_signal = dumper.get_pred()
        dumper.finish_conditions.append((predicate_signal, "executed_wire"))
        return None
    elif intrinsic == Intrinsic.ASSERT:
        dumper.expose('expr', expr.args[0])
        return None
    elif intrinsic == Intrinsic.WAIT_UNTIL:
        cond = dumper.dump_rval(expr.args[0], False)
        final_cond = cond
        dumper.wait_until = final_cond
        return None
    elif intrinsic == Intrinsic.BARRIER:
        return None
    else:
        raise ValueError(f"Unknown block intrinsic: {expr}")


def codegen_wire_assign(dumper, expr: WireAssign) -> Optional[str]:
    """Generate code for wire assign operations."""
    # Annotate external wire assigns so they show up in the generated script
    from ...ir.module import Downstream

    if isinstance(dumper.current_module, Downstream):
        wire = expr.wire
        value = expr.value
        owner = getattr(wire, 'parent', None) or getattr(wire, 'module', None)
        wire_name = getattr(wire, 'name', None)
        if isinstance(owner, ExternalSV) and wire_name:
            dumper.pending_external_inputs[owner].append((wire_name, value))

    return f"# External wire assign: {expr}"


def codegen_wire_read(dumper, expr: WireRead) -> Optional[str]:
    """Generate code for wire read operations."""
    # Document reads from external module outputs and emit the assignment
    dumper.append_code(f'# External wire read: {expr}')
    wire = expr.wire
    owner = getattr(wire, 'parent', None) or getattr(wire, 'module', None)
    wire_name = getattr(wire, 'name', None)
    rval = dumper.dump_rval(expr, False)

    if (
        isinstance(owner, ExternalSV)
        and owner not in dumper.instantiated_external_modules
    ):
        ext_module_name = namify(owner.name)
        inst_name = f"{ext_module_name.lower()}_inst"
        dumper.append_code('# instantiate external module')
        connections = []
        if getattr(owner, 'has_clock', False):
            connections.append('clk=self.clk')
        if getattr(owner, 'has_reset', False):
            connections.append('rst=self.rst')
        for input_name, input_val in dumper.pending_external_inputs.get(owner, []):
            connections.append(f"{input_name}={dumper.dump_rval(input_val, False)}")
        if connections:
            dumper.append_code(f'{inst_name} = {ext_module_name}({", ".join(connections)})')
        else:
            dumper.append_code(f'{inst_name} = {ext_module_name}()')
        dumper.instantiated_external_modules.add(owner)
        dumper.pending_external_inputs.pop(owner, None)

    if owner is not None and wire_name is not None:
        inst_name = f"{namify(owner.name).lower()}_inst"
        return f"{rval} = {inst_name}.{wire_name}"

    return f"# TODO: unresolved external wire read for {expr}"


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
