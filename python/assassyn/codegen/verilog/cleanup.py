"""Post-generation cleanup and signal generation for Verilog codegen."""

from collections import defaultdict
from typing import TYPE_CHECKING, Callable, Dict, List, NamedTuple, Optional, Sequence, TypeVar

from .utils import dump_type, dump_type_cast, get_sram_info

from ...analysis.topo import get_upstreams
from ...ir.module import Downstream
from ...ir.memory.sram import SRAM
from ...ir.array import Slice
from ...ir.memory.base import MemoryBase
from ...ir.const import Const
from ...ir.expr import Expr, FIFOPop, FIFOPush
from ...utils import namify, unwrap_operand

if TYPE_CHECKING:
    from ...ir.expr.array import ArrayRead, ArrayWrite

T = TypeVar("T")


class ValueExposureRender(NamedTuple):
    """Rendered information for a value exposure."""

    exposed_name: str
    dtype_str: str
    rval: str


def _expr_wait_conditions(dumper, expr):
    """Return the wait_until predicates active when *expr* was emitted."""
    if expr is None:
        return ()
    table = getattr(dumper, 'expr_wait_conditions', None)
    if not table:
        return ()
    return tuple(table.get(expr, ()))


def resolve_value_exposure_render(dumper, expr: Expr) -> ValueExposureRender:
    """Compute the rendered name, dtype, and rval for a value exposure."""

    rval = dumper.dump_rval(expr, False)

    exposed_name = dumper.dump_rval(expr, True)
    module_externals = getattr(dumper.current_module, 'externals', [])
    if isinstance(expr, Expr) and expr in module_externals:
        exposed_name = dumper.get_external_port_name(expr)

    if isinstance(expr, Slice):
        left = expr.l.value.value if hasattr(expr.l, 'value') else expr.l
        right = expr.r.value.value if hasattr(expr.r, 'value') else expr.r
        actual_bits = right - left + 1
        dtype_str = f"Bits({actual_bits})"
    else:
        dtype_str = dump_type(expr.dtype)

    return ValueExposureRender(exposed_name=exposed_name, dtype_str=dtype_str, rval=rval)


def generate_sram_control_signals(dumper, sram_info, module_view):
    """Generate control signals for SRAM memory interface."""

    array = sram_info['array']
    writes = list(module_view.writes.get(array, ()))
    reads = list(module_view.reads.get(array, ()))

    if writes:
        first_write = writes[0]
        write_addr = dumper.dump_rval(first_write.idx, False)
        write_pred_literal = dumper.format_predicate(
            getattr(first_write, "meta_cond", None),
            extra_conditions=_expr_wait_conditions(dumper, first_write),
        )
        write_enable = f'executed_wire & ({write_pred_literal})'
        write_data = dumper.dump_rval(first_write.val, False)
    else:
        write_addr = None
        write_enable = 'Bits(1)(0)'
        write_data = f"{dump_type(array.scalar_ty)}(0)"

    read_addr = None
    if reads:
        read_expr = reads[0]
        read_addr = dumper.dump_rval(read_expr.idx, False)

    dumper.append_code(f'self.mem_write_enable = {write_enable}')

    # Address selection (prioritize write address when writing)
    if write_addr and read_addr:
        if write_addr != read_addr:
            dumper.append_code(
                f'self.mem_address = Mux({write_enable}, '
                f'{read_addr}.as_bits(), '
                f'{write_addr}.as_bits())'
            )
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


def _format_reduction_expr(
    predicates: Sequence[str],
    *,
    default_literal: Optional[str],
    op: str = "operator.or_",
) -> str:
    """Format a reduction expression with configurable operator and default literal."""

    if not predicates:
        if default_literal is None:
            raise ValueError("Cannot build predicate reduction without a default literal")
        return default_literal

    if default_literal is None and len(predicates) == 1:
        return predicates[0]

    joined = ", ".join(predicates)
    if default_literal is None:
        return f"reduce({op}, [{joined}])"

    return f"reduce({op}, [{joined}], {default_literal})"


def _emit_predicate_mux_chain(
    entries: Sequence[T],
    *,
    render_predicate: Callable[[T], str],
    render_value: Callable[[T], str],
    default_value: str,
    aggregate_predicates: Callable[[Sequence[str]], str],
) -> tuple[str, str]:
    """Return both the mux chain and aggregate predicate for *entries*."""

    predicate_terms = [render_predicate(entry) for entry in entries]
    aggregate_expr = aggregate_predicates(predicate_terms)

    if not entries:
        return default_value, aggregate_expr

    value_terms = [render_value(entry) for entry in entries]

    if len(value_terms) == 1:
        return value_terms[0], aggregate_expr

    mux_expr = default_value
    for predicate_expr, value_expr in zip(predicate_terms, value_terms):
        mux_expr = f"Mux({predicate_expr}, {mux_expr}, {value_expr})"

    return mux_expr, aggregate_expr


# pylint: disable=too-many-locals,too-many-branches,too-many-statements,too-many-nested-blocks
def cleanup_post_generation(dumper):
    """generating signals for connecting modules"""
    dumper.append_code('')
    completion_guard = "Bits(1)(1)"

    module_metadata = dumper.module_metadata[dumper.current_module]
    module_view = module_metadata.interactions

    if isinstance(dumper.current_module, Downstream):
        node = dumper.current_module
        upstream_modules = sorted(get_upstreams(node), key=lambda mod: mod.name)
        dep_signals = [
            f'self.{namify(dep.name)}_executed'
            for dep in upstream_modules
        ]

        executed_expr = _format_reduction_expr(
            dep_signals,
            default_literal="Bits(1)(0)",
        )
    else:
        # Note: wait_until should NOT gate executed_wire because it should only
        # block operations that come AFTER it in the IR sequence, not ALL operations.
        # Operations before wait_until should still execute.
        exec_conditions = ["self.trigger_counter_pop_valid"]
        completion_guard = dumper.format_predicate(None)

        # If this module consumes from any FIFO(s), treat its execution as an atomic
        # transaction: only "execute" (and thus pop inputs / decrement trigger counter)
        # when all downstream handshakes required by its side-effecting outputs can
        # accept the work. Otherwise, we can drop tokens (pop without push) and
        # desynchronize trigger counters from FIFO occupancy.
        #
        # Producer-only modules (no FIFO pops) are allowed to execute even when some
        # downstream FIFOs are full, because they may update independent state that
        # eventually releases backpressure (gating them can deadlock designs).
        if getattr(module_view, "pops", ()):
            guard_terms = []

            # Guard FIFO pushes (including those implied by async calls / binds).
            fifo_push_groups: Dict[tuple[str, str], list[Expr]] = defaultdict(list)

            for push in getattr(module_view, "pushes", ()):
                if getattr(push, "parent", None) is not dumper.current_module:
                    continue
                fifo = push.fifo
                fifo_push_groups[(namify(fifo.module.name), namify(fifo.name))].append(push)

            for call in getattr(module_metadata, "calls", ()):
                bind = getattr(call, "bind", None)
                if bind is None:
                    continue
                for push in getattr(bind, "pushes", ()):
                    if getattr(push, "parent", None) is not dumper.current_module:
                        continue
                    fifo = push.fifo
                    fifo_push_groups[(namify(fifo.module.name), namify(fifo.name))].append(push)

            for (callee_mod_name, fifo_port_name), pushes in fifo_push_groups.items():
                predicate_terms = []
                for push in pushes:
                    predicate = dumper.format_predicate(
                        getattr(push, "meta_cond", None),
                        extra_conditions=_expr_wait_conditions(dumper, push),
                    )
                    predicate_terms.append(f"({predicate})")
                any_push = _format_reduction_expr(
                    predicate_terms,
                    default_literal="Bits(1)(0)",
                )
                ready = f"self.fifo_{callee_mod_name}_{fifo_port_name}_push_ready"
                guard_terms.append(f"(~({any_push}) | {ready})")

            # Guard async call trigger credits (delta_ready) separately.
            async_groups = dumper.interactions.async_ledger.calls_for_module(dumper.current_module)
            for callee, calls in async_groups.items():
                predicate_terms = []
                for call in calls:
                    predicate = dumper.format_predicate(
                        getattr(call, "meta_cond", None),
                        extra_conditions=_expr_wait_conditions(dumper, call),
                    )
                    predicate_terms.append(f"({predicate})")
                any_call = _format_reduction_expr(
                    predicate_terms,
                    default_literal="Bits(1)(0)",
                )
                callee_ready = f"self.{namify(callee.name)}_trigger_counter_delta_ready"
                guard_terms.append(f"(~({any_call}) | {callee_ready})")

            stall_guard = _format_reduction_expr(
                guard_terms,
                default_literal="Bits(1)(1)",
                op="operator.and_",
            )
            exec_conditions.append(stall_guard)

        executed_expr = _format_reduction_expr(
            exec_conditions,
            default_literal="Bits(1)(1)",
            op="operator.and_",
        )
    dumper.append_code(f"executed_wire = {executed_expr}")

    finish_terms = []
    for finish_site in module_metadata.finish_sites:
        predicate = dumper.format_predicate(
            getattr(finish_site, "meta_cond", None),
            extra_conditions=_expr_wait_conditions(dumper, finish_site),
        )
        finish_terms.append(f"({predicate} & executed_wire)")
    finish_expr = _format_reduction_expr(
        finish_terms,
        default_literal="Bits(1)(0)",
    )
    dumper.append_code(f"self.finish = {finish_expr}")

    if isinstance(dumper.current_module, SRAM):
        sram_info = get_sram_info(dumper.current_module)
        if sram_info:
            generate_sram_control_signals(dumper, sram_info, module_view)

    all_arrays = set(module_view.writes.keys()) | set(module_view.reads.keys())
    for arr in all_arrays:
        module_writes = list(module_view.writes.get(arr, ()))
        owner = arr.owner
        if isinstance(owner, MemoryBase) and arr.is_payload(owner):
            continue

        metadata = dumper.array_metadata.metadata_for(arr)
        if metadata is None:
            continue

        array_name = dumper.dump_rval(arr, False)
        array_dtype = arr.scalar_ty
        array_dtype_str = dump_type(array_dtype)

        if module_writes:
            port_mapping = metadata.write_ports
            port_idx = port_mapping.get(dumper.current_module)
            if port_idx is not None:
                port_suffix = f"_port{port_idx}"

                def render_array_predicate(write: 'ArrayWrite') -> str:
                    return dumper.format_predicate(
                        getattr(write, "meta_cond", None),
                        extra_conditions=_expr_wait_conditions(dumper, write),
                    )

                def render_array_value(
                    write: 'ArrayWrite',
                    dtype_str: str = array_dtype_str,
                    dtype=array_dtype,
                ) -> str:
                    value_expr = dumper.dump_rval(write.val, False)
                    if dump_type(write.val.dtype) != dtype_str:
                        value_expr = f"{value_expr}.{dump_type_cast(dtype)}"
                    return value_expr

                def aggregate_array(predicates: Sequence[str]) -> str:
                    return _format_reduction_expr(predicates, default_literal=None)

                wdata_expr, aggregated_predicates = _emit_predicate_mux_chain(
                    module_writes,
                    render_predicate=render_array_predicate,
                    render_value=render_array_value,
                    default_value=f"{array_dtype_str}(0)",
                    aggregate_predicates=aggregate_array,
                )

                idx_default = f"{dump_type(module_writes[0].idx.dtype)}(0)"

                def render_array_index(write: 'ArrayWrite') -> str:
                    return dumper.dump_rval(write.idx, False)

                def reuse_aggregated(
                    _predicates: Sequence[str],
                    combined: str = aggregated_predicates,
                ) -> str:
                    return combined

                widx_expr, _ = _emit_predicate_mux_chain(
                    module_writes,
                    render_predicate=render_array_predicate,
                    render_value=render_array_index,
                    default_value=idx_default,
                    aggregate_predicates=reuse_aggregated,
                )

                dumper.append_code(
                    f'self.{array_name}_w{port_suffix} = executed_wire & ({aggregated_predicates})'
                )

                dumper.append_code(f'self.{array_name}_wdata{port_suffix} = {wdata_expr}')
                dumper.append_code(f'self.{array_name}_widx{port_suffix} = {widx_expr}.as_bits()')

        module_reads = module_view.reads.get(arr, ())
        if module_reads and arr.index_bits > 0:
            assigned_read_ports = set()
            index_bits = arr.index_bits
            for expr in module_reads:
                port_idx = dumper.array_metadata.read_port_index_for_expr(expr)
                if port_idx is None or port_idx in assigned_read_ports:
                    continue
                idx_value = dumper.dump_rval(expr.idx, False)
                idx_dtype = expr.idx.dtype
                if (
                    hasattr(idx_dtype, 'is_raw')
                    and idx_dtype.is_raw()
                    and idx_dtype.bits == index_bits
                ):
                    cast_idx = idx_value
                else:
                    cast_idx = f'{idx_value}.as_bits({index_bits})'
                dumper.append_code(
                    f'self.{array_name}_ridx_port{port_idx} = {cast_idx}'
                )
                assigned_read_ports.add(port_idx)

    value_groups: Dict[Expr, List[Expr]] = defaultdict(list)
    for expr in module_metadata.value_exposures:
        value_groups[expr].append(expr)

    for expr, grouped_exposures in value_groups.items():
        if isinstance(unwrap_operand(expr), Const):
            continue
        render = resolve_value_exposure_render(dumper, expr)
        dumper.append_code(f'# Expose: {expr}')
        dumper.append_code(f'self.expose_{render.exposed_name} = {render.rval}')
        predicate_terms = []
        for entry in grouped_exposures:
            predicate = getattr(entry, "meta_cond", None)
            if predicate is None:
                continue
            formatted = dumper.format_predicate(
                predicate,
                extra_conditions=_expr_wait_conditions(dumper, entry),
            )
            if formatted == "Bits(1)(1)":
                continue
            predicate_terms.append(f'({formatted})')
        pred_condition = (
            _format_reduction_expr(
                predicate_terms,
                default_literal="Bits(1)(0)",
            )
            if predicate_terms
            else "Bits(1)(1)"
        )
        dumper.append_code(
            f'self.valid_{render.exposed_name} = executed_wire & ({pred_condition})'
        )

    async_groups = dumper.interactions.async_ledger.calls_for_module(dumper.current_module)
    for callee, trigger_entries in async_groups.items():
        rval = dumper.dump_rval(callee, False)
        if not trigger_entries:
            dumper.append_code(f'self.{rval}_trigger = UInt(8)(0)')
            continue

        callee_ready = f"self.{namify(callee.name)}_trigger_counter_delta_ready"
        trigger_predicates = []
        for call in trigger_entries:
            # Async calls enqueue values into the callee’s input FIFOs (one per bound port).
            # The trigger counter delta must remain aligned with FIFO occupancy; otherwise the
            # callee can pop when no data was enqueued (e.g., caller ran while callee FIFO full).
            extra_conditions = list(_expr_wait_conditions(dumper, call))
            extra_conditions.append(callee_ready)

            bind = getattr(call, "bind", None)
            if bind is not None:
                push_ready_terms = {
                    f"self.fifo_{namify(push.fifo.module.name)}_{namify(push.fifo.name)}_push_ready"
                    for push in getattr(bind, "pushes", ())
                    if getattr(push, "parent", None) is dumper.current_module
                }
                extra_conditions.extend(sorted(push_ready_terms))

            trigger_predicates.append(
                dumper.format_predicate(
                    getattr(call, "meta_cond", None),
                    extra_conditions=extra_conditions,
                )
            )
        dumper.append_code(f'# Summing triggers for {rval}')
        add_terms = [f"Mux({pred}, UInt(8)(0), UInt(8)(1))" for pred in trigger_predicates]
        sum_expression = f"reduce(operator.add, [{', '.join(add_terms)}])"
        resized_sum = f"(({sum_expression}).as_bits()[0:8].as_uint())"
        final_trigger_value = f"Mux(executed_wire, UInt(8)(0), {resized_sum})"
        dumper.append_code(f'self.{rval}_trigger = {final_trigger_value}')

    for fifo_port in module_view.fifo_ports:
        interactions = module_view.fifo_map[fifo_port]
        fifo_name = dumper.dump_rval(fifo_port, False)
        local_pushes = [
            entry
            for entry in interactions
            if (
                isinstance(entry, FIFOPush)
                and getattr(entry, "parent", None) is dumper.current_module
            )
        ]
        local_pops = [
            entry
            for entry in interactions
            if (
                isinstance(entry, FIFOPop)
                and getattr(entry, "parent", None) is dumper.current_module
            )
        ]

        if local_pushes:
            fifo_default = f"{dump_type(fifo_port.dtype)}(0)"

            def render_fifo_predicate(entry) -> str:
                return dumper.format_predicate(
                    getattr(entry, "meta_cond", None),
                    extra_conditions=_expr_wait_conditions(dumper, entry),
                    raw=True,
                )

            def render_fifo_value(entry) -> str:
                return dumper.dump_rval(entry.val, False)

            def aggregate_fifo(predicates: Sequence[str]) -> str:
                wrapped = [f"({term})" for term in predicates]
                return _format_reduction_expr(wrapped, default_literal="Bits(1)(0)")

            fifo_data_expr, fifo_predicate_expr = _emit_predicate_mux_chain(
                local_pushes,
                render_predicate=render_fifo_predicate,
                render_value=render_fifo_value,
                default_value=fifo_default,
                aggregate_predicates=aggregate_fifo,
            )

            dumper.append_code(f'# Push logic for port: {fifo_name}')
            ready_signal = (
                f"self.fifo_{namify(fifo_port.module.name)}_{fifo_name}_push_ready"
            )
            fifo_prefix = f"self.{namify(fifo_port.module.name)}_{fifo_name}"

            dumper.append_code(
                f"{fifo_prefix}_push_valid = executed_wire & "
                f"({fifo_predicate_expr}) & {ready_signal}"
            )
            dumper.append_code(f"{fifo_prefix}_push_data = {fifo_data_expr}")

        if local_pops:
            pop_predicates = []
            for entry in local_pops:
                predicate = dumper.format_predicate(
                    getattr(entry, "meta_cond", None),
                    extra_conditions=_expr_wait_conditions(dumper, entry),
                    raw=True,
                )
                pop_predicates.append(f'({predicate})')
            final_pop_condition = _format_reduction_expr(
                pop_predicates,
                default_literal="Bits(1)(0)",
            )
            dumper.append_code(f'# {local_pops[0]}')
            dumper.append_code(
                f"self.{fifo_name}_pop_ready = executed_wire & ({final_pop_condition})"
            )

    external_exposures = dumper.external_output_exposures.get(dumper.current_module, {})
    for data in external_exposures.values():
        output_name = data['output_name']
        index_key = data.get('index_key')
        index_operand = data['index_operand']
        if index_key is None or index_key == ('const', 0):
            source_expr = f"{data['instance_name']}.{data['port_name']}"
        elif index_operand is not None:
            index_code = dumper.dump_rval(index_operand, False)
            source_expr = f"{data['instance_name']}.{data['port_name']}[{index_code}]"
        else:
            source_expr = f"{data['instance_name']}.{data['port_name']}"
        dumper.append_code(f'# External output exposure: {source_expr}')
        dumper.append_code(f'self.expose_{output_name} = {source_expr}')
        # Include the condition predicate for the valid signal if available
        condition = data.get('condition', 'Bits(1)(1)')
        dumper.append_code(f'self.valid_{output_name} = executed_wire & ({condition})')

    dumper.append_code(f'self.executed = executed_wire & ({completion_guard})')
