# pylint: disable=too-many-locals,too-many-statements,too-many-branches
"""Intrinsic operations code generation for Verilog.

This module contains functions to generate Verilog code for intrinsic operations
including PureIntrinsic, Intrinsic, and Log.
"""

from typing import Optional, TYPE_CHECKING
from string import Formatter

from ....ir.expr import Log
from ....ir.expr.intrinsic import PureIntrinsic, Intrinsic, ExternalIntrinsic
from ....ir.const import Const
from ....ir.dtype import Int
from ....utils import unwrap_operand, namify

if TYPE_CHECKING:
    from ..design import CIRCTDumper


def codegen_log(dumper, expr: Log) -> Optional[str]:
    """Generate code for log operations."""
    formatter_str = expr.operands[0].value

    arg_print_snippets = []
    condition_snippets = []
    module_name = namify(dumper.current_module.name)

    final_conditions = []
    seen_conditions = set()

    def append_condition(cond: Optional[str]):
        if cond and cond not in seen_conditions:
            seen_conditions.add(cond)
            final_conditions.append(cond)

    def _sanitize(name: str) -> str:
        if name.startswith("self."):
            name = name[5:]
        return name.replace(".", "_")

    meta_cond = expr.meta_cond
    if meta_cond is None:
        raise ValueError("Log.meta_cond is unexpectedly missing")

    if isinstance(meta_cond, Const):
        if meta_cond.value == 0:
            append_condition('False')
    else:
        exposed_name = _sanitize(dumper.dump_rval(meta_cond, True))
        valid_signal = f'dut.{module_name}.valid_{exposed_name}.value'
        expose_signal = f'dut.{module_name}.expose_{exposed_name}.value'
        append_condition(f'({valid_signal} & {expose_signal})')

    for i in expr.operands[1:]:
        operand = unwrap_operand(i)
        if not isinstance(operand, Const):
            exposed_name = _sanitize(dumper.dump_rval(operand, True))
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

    if condition_snippets:
        append_condition(" and ".join(condition_snippets))

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


def _handle_fifo_intrinsic(dumper, expr, intrinsic, rval):
    """Handle FIFO_VALID and FIFO_PEEK intrinsics."""
    if intrinsic not in (PureIntrinsic.FIFO_VALID, PureIntrinsic.FIFO_PEEK):
        return None

    fifo = expr.args[0]
    fifo_name = dumper.dump_rval(fifo, False)
    if intrinsic == PureIntrinsic.FIFO_PEEK:
        return f'{rval} = self.{fifo_name}'
    return f'{rval} = self.{fifo_name}_valid'


def _handle_value_valid(dumper, expr, intrinsic, rval):
    """Handle VALUE_VALID intrinsic."""
    if intrinsic != PureIntrinsic.VALUE_VALID:
        return None

    value_expr = expr.operands[0].value
    if value_expr.parent != expr.parent:
        port_name = dumper.get_external_port_name(value_expr)
        return f"{rval} = self.{port_name}_valid"
    return f"{rval} = self.executed"


def _handle_external_output(dumper, expr, intrinsic, rval):
    """Handle reads from external module outputs."""
    if intrinsic != PureIntrinsic.EXTERNAL_OUTPUT_READ:
        return None

    instance_operand = expr.args[0]  # Operand wrapping the ExternalIntrinsic
    instance = unwrap_operand(instance_operand)
    port_operand = expr.args[1]
    port_name = port_operand.value if hasattr(port_operand, 'value') else port_operand
    index_operand = expr.args[2] if len(expr.args) > 2 else None

    result = None
    instance_owner = dumper.external_metadata.owner_for(instance)
    if instance_owner and instance_owner != dumper.current_module:
        # Cross-module access: use the exposed port value provided on inputs.
        port_name_for_read = dumper.get_external_port_name(expr)
        wire_key = dumper.get_external_wire_key(instance, port_name, index_operand)
        assignment_key = (dumper.current_module, wire_key)
        if assignment_key not in dumper.external_wire_assignment_keys:
            dumper.external_wire_assignment_keys.add(assignment_key)
            dumper.external_wire_assignments.append({
                'consumer': dumper.current_module,
                'producer': instance_owner,
                'expr': expr,
                'wire': wire_key,
            })
        result = f"{rval} = self.{port_name_for_read}"
    else:
        inst_name = dumper.external_instance_names.get(instance)
        if inst_name is None:
            inst_name = dumper.dump_rval(instance, False)
            dumper.external_instance_names[instance] = inst_name

        port_specs = instance.external_class.port_specs()
        wire_spec = port_specs.get(port_name)

        if wire_spec is not None and wire_spec.kind == 'reg':
            if index_operand is None:
                result = f"{rval} = {inst_name}.{port_name}"
            else:
                idx_operand = unwrap_operand(index_operand)
                if isinstance(idx_operand, Const) and idx_operand.value == 0:
                    result = f"{rval} = {inst_name}.{port_name}"
                else:
                    index_code = dumper.dump_rval(index_operand, False)
                    result = f"{rval} = {inst_name}.{port_name}[{index_code}]"
        else:
            if index_operand is None:
                result = f"{rval} = {inst_name}.{port_name}"
            else:
                index_code = dumper.dump_rval(index_operand, False)
                result = f"{rval} = {inst_name}.{port_name}[{index_code}]"

    return result


def codegen_pure_intrinsic(dumper, expr: PureIntrinsic) -> Optional[str]:
    """Generate code for pure intrinsic operations."""
    intrinsic = expr.opcode
    rval = dumper.dump_rval(expr, False)

    # Handle current_cycle directly
    if intrinsic == PureIntrinsic.CURRENT_CYCLE:
        return f"{rval} = self.cycle_count"

    for handler in (_handle_fifo_intrinsic, _handle_value_valid, _handle_external_output):
        result = handler(dumper, expr, intrinsic, rval)
        if result is not None:
            return result

    raise ValueError(f"Unknown intrinsic: {expr}")


def codegen_external_intrinsic(dumper, expr: ExternalIntrinsic) -> Optional[str]:
    """Generate Verilog for external module instantiation.

    For now, we don't generate inline Verilog instantiation.
    External modules will be handled separately through the external module system.
    """
    rval = dumper.dump_rval(expr, False)
    ext_class = expr.external_class
    metadata = ext_class.metadata()
    wrapper_name = dumper.external_wrapper_names.get(ext_class)
    if wrapper_name is None:
        wrapper_name = f"{ext_class.__name__}_ffi"
        dumper.external_wrapper_names[ext_class] = wrapper_name

    connections = []
    if metadata.get('has_clock'):
        connections.append('clk=self.clk')
    if metadata.get('has_reset'):
        connections.append('rst=self.rst')
    for port_name, value in expr.input_connections.items():
        value_code = dumper.dump_rval(value, False)
        connections.append(f"{port_name}={value_code}")

    call = f"{wrapper_name}({', '.join(connections)})" if connections else f"{wrapper_name}()"
    dumper.external_instance_names[expr] = rval

    entries = dumper.external_metadata.reads_for_instance(expr)
    if entries:
        exposures = dumper.external_output_exposures[dumper.current_module]
        seen_keys = set()
        for entry in entries:
            wire_key = dumper.get_external_wire_key(
                expr,
                entry.port_name,
                entry.index_operand,
            )
            if wire_key in seen_keys:
                continue
            seen_keys.add(wire_key)
            output_name = f"{rval}_{entry.port_name}"
            dumper.external_wire_outputs[wire_key] = output_name
            current_pred = dumper.get_pred(expr)
            exposures.setdefault(wire_key, {
                'output_name': output_name,
                'dtype': entry.expr.dtype,
                'instance_name': rval,
                'port_name': entry.port_name,
                'index_operand': entry.index_operand,
                'index_key': wire_key[2],
                'condition': current_pred,
            })


    return f"{rval} = {call}"


def codegen_intrinsic(dumper, expr: Intrinsic) -> Optional[str]:
    """Generate code for intrinsic operations."""
    # Check if this is an ExternalIntrinsic first
    if isinstance(expr, ExternalIntrinsic):
        return codegen_external_intrinsic(dumper, expr)

    intrinsic = expr.opcode

    if intrinsic == Intrinsic.FINISH:
        return None
    if intrinsic == Intrinsic.ASSERT:
        return None
    if intrinsic == Intrinsic.WAIT_UNTIL:
        cond = dumper.dump_rval(expr.args[0], False)
        final_cond = cond
        dumper.wait_until = final_cond
        return None
    if intrinsic == Intrinsic.PUSH_CONDITION:
        return None
    if intrinsic == Intrinsic.POP_CONDITION:
        return None
    if intrinsic == Intrinsic.EXTERNAL_INSTANTIATE:
        # Should be handled by ExternalIntrinsic check above
        raise RuntimeError("EXTERNAL_INSTANTIATE should be handled by ExternalIntrinsic")

    raise ValueError(f"Unknown block intrinsic: {expr}")
