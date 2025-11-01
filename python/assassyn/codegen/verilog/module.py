"""Module port generation utilities for Verilog code generation."""

from .cleanup import resolve_value_exposure_render
from .utils import dump_type, get_sram_info
from ...analysis.topo import get_upstreams
from ...ir.module import Module, Downstream
from ...ir.memory.sram import SRAM
from ...ir.module.base import ModuleBase
from ...ir.expr import Bind, Expr
from ...ir.expr.intrinsic import ExternalIntrinsic
from ...ir.const import Const
from ...utils import namify, unwrap_operand


# pylint: disable=too-many-arguments,too-many-locals,too-many-branches,too-many-statements
def generate_module_ports(dumper, node: Module) -> None:
    """Generate port declarations for a module.

    Args:
        dumper: The CIRCTDumper instance
        node: The module to generate ports for
    """
    is_downstream = isinstance(node, Downstream)
    is_sram = isinstance(node, SRAM)
    async_callers = list(dumper.async_callers(node))
    is_driver = not async_callers

    module_metadata = dumper.module_metadata[node]
    module_view = module_metadata.interactions
    pushes = list(module_view.pushes)
    pops = list(module_view.pops)
    calls = list(module_metadata.calls)

    dumper.append_code('clk = Clock()')
    dumper.append_code('rst = Reset()')
    dumper.append_code('executed = Output(Bits(1))')
    dumper.append_code('cycle_count = Input(UInt(64))')
    dumper.append_code('finish = Output(Bits(1))')

    if is_downstream:
        upstream_modules = sorted(get_upstreams(node), key=lambda mod: mod.name)
        for dep_mod in upstream_modules:
            dumper.append_code(f'{namify(dep_mod.name)}_executed = Input(Bits(1))')
        if is_sram:
            sram_info = get_sram_info(node)
            if sram_info:
                sram_array = sram_info['array']
                dumper.append_code(f'mem_dataout = Input({dump_type(sram_array.scalar_ty)})')
                index_bits = sram_array.index_bits if sram_array.index_bits > 0 else 1
                dumper.append_code(f'mem_address = Output(Bits({index_bits}))')
                dumper.append_code(f'mem_write_data = Output({dump_type(sram_array.scalar_ty)})')
                dumper.append_code('mem_write_enable = Output(Bits(1))')
                dumper.append_code('mem_read_enable = Output(Bits(1))')

    elif is_driver or async_callers:
        dumper.append_code('trigger_counter_pop_valid = Input(Bits(1))')

    added_external_ports = set()

    consumer_entries = dumper.external_metadata.reads_for_consumer(node)
    for entry in consumer_entries:
        port_name = dumper.get_external_port_name(entry.expr)
        if port_name in added_external_ports:
            continue
        dtype = dump_type(entry.expr.dtype)
        dumper.append_code(f'{port_name} = Input({dtype})')
        dumper.append_code(f'{port_name}_valid = Input(Bits(1))')
        added_external_ports.add(port_name)

    for ext_val in node.externals:
        if isinstance(ext_val, (Bind, ExternalIntrinsic)) or isinstance(
                unwrap_operand(ext_val), Const):
            continue
        port_name = dumper.get_external_port_name(ext_val)
        parent_ref = getattr(ext_val, 'parent', None)
        if isinstance(parent_ref, ModuleBase):
            parent_module = parent_ref
        else:
            parent_module = getattr(parent_ref, 'module', None)
        print(
            f"[verilog] module {node.name} external port {port_name} "
            f"from {parent_module} expr={ext_val}"
        )
        if port_name in added_external_ports:
            continue
        port_type = dump_type(ext_val.dtype)
        dumper.append_code(f'{port_name} = Input({port_type})')
        dumper.append_code(f'{port_name}_valid = Input(Bits(1))')
        added_external_ports.add(port_name)

    if not is_downstream:
        for i in node.ports:
            name = namify(i.name)
            dumper.append_code(f'{name} = Input({dump_type(i.dtype)})')
            dumper.append_code(f'{name}_valid = Input(Bits(1))')
            popped_fifos = {p.fifo for p in pops}
            has_pop = i in popped_fifos
            if has_pop:
                dumper.append_code(f'{name}_pop_ready = Output(Bits(1))')

    unique_push_handshake_targets = {(p.fifo.module, p.fifo.name) for p in pushes}
    unique_call_handshake_targets = {c.bind.callee for c in calls}
    unique_output_push_ports = {p.fifo for p in pushes}

    for module, fifo_name in unique_push_handshake_targets:
        port_name = f'fifo_{namify(module.name)}_{namify(fifo_name)}_push_ready'
        dumper.append_code(f'{port_name} = Input(Bits(1))')
    for callee in unique_call_handshake_targets:
        port_name = f'{namify(callee.name)}_trigger_counter_delta_ready'
        dumper.append_code(f'{port_name} = Input(Bits(1))')

    # Output push ports for async callees and FIFO producers
    for fifo_port in unique_output_push_ports:
        port_prefix = f"{namify(fifo_port.module.name)}_{namify(fifo_port.name)}"
        dumper.append_code(f'{port_prefix}_push_valid = Output(Bits(1))')
        dtype = fifo_port.dtype
        dumper.append_code(f'{port_prefix}_push_data = Output({dump_type(dtype)})')
    for callee in unique_call_handshake_targets:
        dumper.append_code(f'{namify(callee.name)}_trigger = Output(UInt(8))')

    # pylint: disable=too-many-nested-blocks
    for arr_container in dumper.sys.arrays:
        arr = arr_container
        if is_sram:
            sram_info = get_sram_info(node)
            if sram_info and arr == sram_info['array']:
                continue
        metadata = dumper.array_metadata.metadata_for(arr)
        if metadata is None:
            continue

        users = dumper.array_metadata.users_for(arr)
        if not any(user is node for user in users):
            continue

        read_port_indices = dumper.array_metadata.read_port_indices(arr, node)
        if not read_port_indices:
            for module_key, ports in metadata.read_ports_by_module.items():
                if module_key is node:
                    read_port_indices = ports
                    break

        writes_idx = dumper.array_metadata.write_port_index(arr, node)
        if writes_idx is None:
            for module_key, idx in metadata.write_ports.items():
                if module_key is node:
                    writes_idx = idx
                    break

        if read_port_indices or writes_idx is not None:
            index_bits = arr.index_bits
            idx_type = index_bits if index_bits > 0 else 1
            for port_idx in read_port_indices:
                port_suffix = f"_port{port_idx}"
                if index_bits > 0:
                    dumper.append_code(
                        f'{namify(arr.name)}_ridx{port_suffix} = Output(Bits({idx_type}))'
                    )
                dumper.append_code(
                    f'{namify(arr.name)}_rdata{port_suffix} = '
                    f'Input({dump_type(arr.scalar_ty)})'
                )
            if writes_idx is not None:
                port_suffix = f"_port{writes_idx}"
                dumper.append_code(
                    f'{namify(arr.name)}_w{port_suffix} = Output(Bits(1))'
                )
                dumper.append_code(
                    f'{namify(arr.name)}_wdata{port_suffix} ='
                    f' Output({dump_type(arr.scalar_ty)})'
                )
                dumper.append_code(
                    f'{namify(arr.name)}_widx{port_suffix} ='
                    f' Output(Bits({idx_type}))'
                )

    ordered_exposures: list[Expr] = []
    seen_ids: set[int] = set()
    for expr in module_metadata.value_exposures:
        expr_id = id(expr)
        if expr_id in seen_ids:
            continue
        seen_ids.add(expr_id)
        ordered_exposures.append(expr)

    for expr in ordered_exposures:
        render = resolve_value_exposure_render(dumper, expr)
        dumper.append_code(f'expose_{render.exposed_name} = Output({render.dtype_str})')
        dumper.append_code(f'valid_{render.exposed_name} = Output(Bits(1))')

    external_exposures = dumper.external_output_exposures.get(node, {})
    for data in external_exposures.values():
        output_name = data['output_name']
        dtype_str = dump_type(data['dtype'])
        dumper.append_code(f'expose_{output_name} = Output({dtype_str})')
        dumper.append_code(f'valid_{output_name} = Output(Bits(1))')
