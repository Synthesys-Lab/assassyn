# pylint: disable=too-many-locals,too-many-statements,too-many-branches
"""Intrinsic operations code generation for Verilog.

This module contains functions to generate Verilog code for intrinsic operations
including PureIntrinsic, Intrinsic, and Log.
"""

from typing import Optional
from string import Formatter

from ....ir.expr import Log
from ....ir.expr.intrinsic import PureIntrinsic, Intrinsic
from ....ir.const import Const
from ....ir.dtype import Int
from ....ir.block import CondBlock, CycledBlock
from ....utils import unwrap_operand, namify


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
    if intrinsic == PureIntrinsic.VALUE_VALID:
        value_expr = expr.operands[0].value
        if value_expr.parent.module != expr.parent.module:
            port_name = dumper.get_external_port_name(value_expr)
            return f"{rval} = self.{port_name}_valid"
        return f"{rval} = self.executed"

    raise ValueError(f"Unknown intrinsic: {expr}")


def codegen_intrinsic(dumper, expr: Intrinsic) -> Optional[str]:
    """Generate code for intrinsic operations."""
    intrinsic = expr.opcode

    if intrinsic == Intrinsic.FINISH:
        predicate_signal = dumper.get_pred()
        dumper.finish_conditions.append((predicate_signal, "executed_wire"))
        return None
    if intrinsic == Intrinsic.ASSERT:
        dumper.expose('expr', expr.args[0])
        return None
    if intrinsic == Intrinsic.WAIT_UNTIL:
        cond = dumper.dump_rval(expr.args[0], False)
        final_cond = cond
        dumper.wait_until = final_cond
        return None
    if intrinsic == Intrinsic.BARRIER:
        return None

    raise ValueError(f"Unknown block intrinsic: {expr}")
