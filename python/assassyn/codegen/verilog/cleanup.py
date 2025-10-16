"""Post-generation cleanup and signal generation for Verilog codegen."""

from .utils import (
    dump_type,
    dump_type_cast,
    get_sram_info,
)

from ...ir.module import Downstream, Module, Port, Wire
from ...ir.memory.sram import SRAM
from ...ir.array import Array, Slice
from ...ir.const import Const
from ...ir.expr import (
    Expr,
    ArrayWrite,
    ArrayRead,
    FIFOPush,
    FIFOPop,
    AsyncCall,
)
from ...utils import namify, unwrap_operand


# pylint: disable=too-many-locals,too-many-branches
def generate_sram_control_signals(dumper, sram_info):
    """Generate control signals for SRAM memory interface."""
    array = sram_info['array']

    array_writes = []
    array_reads = []
    write_addr = None
    write_data = None
    read_addr = None

    for key, exposes in dumper._exposes.items():  # pylint: disable=protected-access
        if isinstance(key, Array) and key == array:
            for expr, pred in exposes:
                if isinstance(expr, ArrayWrite):
                    array_writes.append((expr, pred))
                elif isinstance(expr, ArrayRead):
                    array_reads.append((expr, pred))

    if array_writes:
        write_expr, write_pred = array_writes[0]
        write_addr = dumper.dump_rval(write_expr.idx, False)
        write_enable = f'executed_wire & ({write_pred})'
        write_data = dumper.dump_rval(write_expr.val, False)
    else:
        write_enable = 'Bits(1)(0)'
        write_addr = None
        write_data = dump_type(array.scalar_ty)(0)
    read_addr = None
    if array_reads:
        read_expr, _ = array_reads[0]
        read_addr = dumper.dump_rval(read_expr.idx, False)

    dumper.append_code(f'self.mem_write_enable = {write_enable}')

    # Address selection (prioritize write address when writing)
    if write_addr and read_addr:
        if write_addr != read_addr:
            dumper.append_code(f'self.mem_address = Mux({write_enable},'
                f' {read_addr}.as_bits(), {write_addr}.as_bits())')
        else:
            dumper.append_code(f'self.mem_address = {write_addr}.as_bits()')
    elif write_addr:
        dumper.append_code(f'self.mem_address = {write_addr}.as_bits()')
    elif read_addr:
        dumper.append_code(f'self.mem_address = {read_addr}.as_bits()')
    else:
        dumper.append_code(f'self.mem_address = Bits({array.index_bits})(0)')


    dumper.append_code(f'self.mem_write_data = {write_data}')

    dumper.append_code('self.mem_read_enable = Bits(1)(1)')  # Always enable reads


def build_mux_chain(dumper, writes, dtype):
    """Helper to build a mux chain for write data"""
    first_val = dumper.dump_rval(writes[0][0].val, False)
    if writes[0][0].val.dtype != dump_type(dtype):
        first_val = f"{first_val}.{dump_type_cast(dtype)}"
    mux = f"Mux({writes[0][1]}, {dump_type(dtype)}(0), {first_val})"

    for expr, pred in writes[1:]:
        val = dumper.dump_rval(expr.val, False)
        if expr.val.dtype != dump_type(dtype):
            val = f"{val}.{dump_type_cast(dtype)}"
        mux = f"Mux({pred}, {mux}, {val})"

    return mux


# pylint: disable=too-many-locals,too-many-branches,too-many-statements,too-many-nested-blocks
def cleanup_post_generation(dumper):
    """generating signals for connecting modules"""
    dumper.append_code('')

    exec_conditions = []
    if isinstance(dumper.current_module, Downstream):
        node = dumper.current_module
        if dumper.current_module in dumper.downstream_dependencies:
            dep_signals = [f'self.{namify(dep.name)}_executed'
                for dep in dumper.downstream_dependencies[node]]
            if dep_signals:
                dumper.append_code(f"executed_wire = ({' | '.join(dep_signals)})")
            else:
                dumper.append_code('executed_wire = Bits(1)(0)')
        else:
            dumper.append_code('executed_wire = Bits(1)(0)')
    else:
        exec_conditions.append("self.trigger_counter_pop_valid")
        if dumper.wait_until:
            exec_conditions.append(f"({dumper.wait_until})")

        if not exec_conditions:
            dumper.append_code('executed_wire = Bits(1)(1)')
        else:
            dumper.append_code(f"executed_wire = {' & '.join(exec_conditions)}")

    if dumper.finish_conditions:
        finish_terms = []
        for pred, exec_signal in dumper.finish_conditions:
            finish_terms.append(f"({pred} & {exec_signal})")

        if len(finish_terms) == 1:
            dumper.append_code(f'self.finish = {finish_terms[0]}')
        else:
            dumper.append_code(f'self.finish = {" | ".join(finish_terms)}')
    else:
        dumper.append_code('self.finish = Bits(1)(0)')

    if isinstance(dumper.current_module, SRAM):
        sram_info = get_sram_info(dumper.current_module)
        if sram_info:
            generate_sram_control_signals(dumper, sram_info)
    # pylint: disable=too-many-nested-blocks
    for key, exposes in dumper._exposes.items():  # pylint: disable=protected-access
        if isinstance(key, Array):
            if key in dumper.sram_payload_arrays:
                continue
            array_writes = [
                    (e, p) for e, p in exposes
                    if isinstance(e, ArrayWrite)
                ]
            array_reads = [
                    (e, p) for e, p in exposes
                    if isinstance(e, ArrayRead)
                ]
            arr = key
            array_name = dumper.dump_rval(arr, False)
            array_dtype = arr.scalar_ty
            port_mapping = dumper.array_write_port_mapping.get(arr, {})
            # Group writes by their source module
            writes_by_module = {}
            for expr, pred in array_writes:
                module = expr.module
                if module not in writes_by_module:
                    writes_by_module[module] = []
                writes_by_module[module].append((expr, pred))
            # Generate signals for each port
            for module, module_writes in writes_by_module.items():
                port_idx = port_mapping[module]
                port_suffix = f"_port{port_idx}"
                # Write enable
                ce_terms = [p for _, p in module_writes]
                dumper.append_code(
                    f'self.{array_name}_w{port_suffix} = '
                    f'executed_wire & ({" | ".join(ce_terms)})'
                )
                # Write data (mux if multiple writes from same module)
                if len(module_writes) == 1:
                    wdata = dumper.dump_rval(module_writes[0][0].val, False)
                    if module_writes[0][0].val.dtype != dump_type(array_dtype):
                        wdata = f"{wdata}.{dump_type_cast(array_dtype)}"
                else:
                    # Build mux chain
                    wdata = build_mux_chain(dumper, module_writes, array_dtype)
                dumper.append_code(f'self.{array_name}_wdata{port_suffix} = {wdata}')
                if len(module_writes) == 1:
                    # Single write - no mux needed, just use the index directly
                    widx_mux = dumper.dump_rval(module_writes[0][0].idx, False)
                else:
                    # Multiple writes - build mux chain
                    widx_mux = (
                        f"Mux({module_writes[0][1]},"
                        f" {dump_type(module_writes[0][0].idx.dtype)}(0),"
                        f" {dumper.dump_rval(module_writes[0][0].idx, False)})"
                    )
                    for expr, pred in module_writes[1:]:
                        widx_mux = f"Mux({pred},  {widx_mux},{dumper.dump_rval(expr.idx, False)})"
                dumper.append_code(
                    f'self.{array_name}_widx{port_suffix} = {widx_mux}.as_bits()'
                    )
            if array_reads and arr.index_bits > 0:
                assigned_read_ports = set()
                index_bits = arr.index_bits
                for expr, _ in array_reads:
                    port_idx = dumper.array_read_expr_port.get(expr)
                    if port_idx is None or port_idx in assigned_read_ports:
                        continue
                    idx_value = dumper.dump_rval(expr.idx, False)
                    idx_dtype = expr.idx.dtype
                    if idx_dtype.is_raw() and idx_dtype.bits == index_bits:
                        cast_idx = idx_value
                    else:
                        cast_idx = f'{idx_value}.as_bits({index_bits})'
                    dumper.append_code(
                        f'self.{array_name}_ridx_port{port_idx} = {cast_idx}'
                    )
                    assigned_read_ports.add(port_idx)

        elif isinstance(key, Port):
            has_push = any(isinstance(e, FIFOPush) for e, p in exposes)
            has_pop = any(isinstance(e, FIFOPop) for e, p in exposes)

            if has_push:
                fifo = dumper.dump_rval(key, False)
                pushes = [(e, p) for e, p in exposes if isinstance(e, FIFOPush)]
                final_push_predicate = " | ".join([f"({p})" for _, p in pushes]) \
                if pushes else "Bits(1)(0)"

                if len(pushes) == 1:
                    final_push_data = dumper.dump_rval(pushes[0][0].val, False)
                else:
                    mux_data = f"{dump_type(key.dtype)}(0)"
                    for expr, pred in pushes:
                        rval = dumper.dump_rval(expr.val, False)
                        mux_data = f"Mux({pred}, {mux_data}, {rval})"
                    final_push_data = mux_data

                dumper.append_code(f'# Push logic for port: {fifo}')
                ready_signal = f"self.fifo_{namify(key.module.name)}_{fifo}_push_ready"

                fifo_prefix = f"self.{namify(key.module.name)}_{fifo}"

                dumper.append_code(
                    f"{fifo_prefix}_push_valid = executed_wire & "
                    f"({final_push_predicate}) & {ready_signal}"
                )
                dumper.append_code(f"{fifo_prefix}_push_data = {final_push_data}")

            if has_pop:
                fifo = dumper.dump_rval(key, False)

                pop_expr = [e for e, p in exposes if isinstance(e, FIFOPop)][0]
                dumper.append_code(f'# {pop_expr}')
                pop_predicates = [pred for expr, pred in exposes if isinstance(expr, FIFOPop)]

                if pop_predicates:
                    final_pop_condition = " | ".join([f"({p})" for p in pop_predicates])
                else:
                    final_pop_condition = "Bits(1)(0)"
                dumper.append_code(
                    f"self.{fifo}_pop_ready = executed_wire & ({final_pop_condition})"
                )

        elif isinstance(key, Module):
            rval = dumper.dump_rval(key, False)

            call_predicates = [pred for expr, pred in exposes if isinstance(expr, AsyncCall)]

            if not call_predicates:
                dumper.append_code(f'self.{rval}_trigger = UInt(8)(0)')
                continue

            dumper.append_code(f'# Summing triggers for {rval}')

            add_terms = [f"Mux({pred}, UInt(8)(0), UInt(8)(1))" for pred in call_predicates]

            if len(add_terms) == 1:
                sum_expression = add_terms[0]
            else:
                sum_expression = f"({' + '.join(add_terms)})"

            resized_sum = f"(({sum_expression}).as_bits()[0:8].as_uint())"
            final_trigger_value = f"Mux(executed_wire, UInt(8)(0), {resized_sum})"
            dumper.append_code(f'self.{rval}_trigger = {final_trigger_value}')

        else:
            expr, pred = exposes[0]
            if isinstance(unwrap_operand(expr), Const):
                continue
            rval = dumper.dump_rval(expr, False)
            if (
                isinstance(expr, Expr)
                and hasattr(dumper.current_module, "externals")
                and expr in dumper.current_module.externals
            ):
                exposed_name = dumper.get_external_port_name(expr)
            else:
                exposed_name = dumper.dump_rval(expr, True)
            if not isinstance(key,ArrayWrite ):
                dtype_str = dump_type(expr.dtype)
            else :
                dtype_str = dump_type(expr.x.dtype)

            if isinstance(expr, Slice):
                # For slice expressions, calculate actual width
                l = expr.l.value.value if hasattr(expr.l, 'value') else expr.l
                r = expr.r.value.value if hasattr(expr.r, 'value') else expr.r
                actual_bits = r - l + 1
                dtype_str = f"Bits({actual_bits})"


            # Special handling for Wire objects - they don't need exposed ports
            if isinstance(key, Wire):
                continue
            # Add port declaration strings to our list
            dumper.exposed_ports_to_add.append(f'expose_{exposed_name} = Output({dtype_str})')
            dumper.exposed_ports_to_add.append(f'valid_{exposed_name} = Output(Bits(1))')

            # Generate the logic assignment
            dumper.append_code(f'# Expose: {expr}')
            dumper.append_code(f'self.expose_{exposed_name} = {rval}')
            dumper.append_code(f'self.valid_{exposed_name} = executed_wire')

    dumper.append_code('self.executed = executed_wire')
