# pylint: disable=C0302
# pylint: disable=protected-access
"""Top-level harness generation for Verilog designs."""

from collections import defaultdict
from typing import TYPE_CHECKING, Any

from .utils import (
    dump_type,
    dump_type_cast,
    get_sram_info,
)

from ...analysis import topo_downstream_modules, get_upstreams
from ...ir.memory.base import MemoryBase
from ...ir.module import Downstream
from ...ir.module.base import ModuleBase
from ...ir.memory.sram import SRAM
from ...ir.expr import (
    Bind,
)
from ...ir.expr.intrinsic import ExternalIntrinsic
from ...ir.dtype import Record
from ...utils import namify, unwrap_operand
from ...ir.const import Const

if TYPE_CHECKING:
    from .design import CIRCTDumper
else:
    CIRCTDumper = Any  # type: ignore

# pylint: disable=too-many-locals,too-many-branches,too-many-statements
def generate_top_harness(dumper: CIRCTDumper):
    """
    Generates a generic Top-level harness that connects all modules based on
    the analyzed dependencies (async calls, array usage).
    """

    dumper.append_code('class Top(Module):')
    dumper.indent += 4
    dumper.append_code('clk = Clock()')
    dumper.append_code('rst = Reset()')
    dumper.append_code('global_cycle_count = Output(UInt(64))')
    dumper.append_code('global_finish = Output(Bits(1))')
    dumper.append_code('')
    dumper.append_code('@generator')
    dumper.append_code('def construct(self):')
    dumper.indent += 4

    sram_modules = [m for m in dumper.sys.downstreams if isinstance(m, SRAM)]
    if sram_modules:
        dumper.append_code('\n# --- SRAM Memory Blackbox Instances ---')
        for data_width, addr_width, array_name in dumper.memory_defs:
            dumper.append_code(f'mem_{array_name}_dataout = Wire(Bits({data_width}))')
            dumper.append_code(f'mem_{array_name}_address = Wire(Bits({addr_width}))')
            dumper.append_code(f'mem_{array_name}_write_data = Wire(Bits({data_width}))')
            dumper.append_code(f'mem_{array_name}_write_enable = Wire(Bits(1))')
            dumper.append_code(f'mem_{array_name}_read_enable = Wire(Bits(1))')

            # Instantiate memory blackbox (as external Verilog module)
            dumper.append_code('# Instantiate memory blackbox module')
            dumper.append_code(
                f'mem_{array_name}_inst = sramBlackbox_{array_name}()'
                '(clk=self.clk, rst_n=~self.rst, '
                f'address=mem_{array_name}_address, '
                f'wd=mem_{array_name}_write_data, '
                'banksel=Bits(1)(1), '
                f'read=mem_{array_name}_read_enable, '
                f'write=mem_{array_name}_write_enable)'
            )

            # Now mem_{array_name}_dataout is properly driven by the module output
            dumper.append_code(f'mem_{array_name}_dataout.assign(mem_{array_name}_inst.dataout)')
            dumper.append_code('')

    dumper.append_code('\n# --- Global Cycle Counter ---')
    dumper.append_code('# A free-running counter for testbench control')

    dumper.append_code('cycle_count = Reg(UInt(64), clk=self.clk, rst=self.rst, rst_value=0)')
    dumper.append_code(
        'cycle_count.assign( (cycle_count + UInt(64)(1)).as_bits()[0:64].as_uint() )'
        )
    dumper.append_code('self.global_cycle_count = cycle_count')

    # --- 1. Wire Declarations (Generic) ---
    dumper.append_code('# --- Wires for FIFOs, Triggers, and Arrays ---')
    for module in dumper.sys.modules:
        for port in module.ports:
            fifo_base_name = f'fifo_{namify(module.name)}_{namify(port.name)}'
            dumper.append_code(f'# Wires for FIFO connected to {module.name}.{port.name}')
            dumper.append_code(f'{fifo_base_name}_push_valid = Wire(Bits(1))')
            dumper.append_code(f'{fifo_base_name}_push_data = Wire(Bits({port.dtype.bits}))')
            dumper.append_code(f'{fifo_base_name}_push_ready = Wire(Bits(1))')
            dumper.append_code(f'{fifo_base_name}_pop_valid = Wire(Bits(1))')
            dumper.append_code(f'{fifo_base_name}_pop_data = Wire(Bits({port.dtype.bits}))')
            dumper.append_code(f'{fifo_base_name}_pop_ready = Wire(Bits(1))')

    # Wires for TriggerCounters (one per module)
    for module in dumper.sys.modules:
        tc_base_name = f'{namify(module.name)}_trigger_counter'
        dumper.append_code(f'# Wires for {module.name}\'s TriggerCounter')
        dumper.append_code(f'{tc_base_name}_delta = Wire(Bits(8))')
        dumper.append_code(f'{tc_base_name}_delta_ready = Wire(Bits(1))')
        dumper.append_code(f'{tc_base_name}_pop_valid = Wire(Bits(1))')
        dumper.append_code(f'{tc_base_name}_pop_ready = Wire(Bits(1))')

    for arr_container in dumper.sys.arrays:
        arr = arr_container
        if arr.is_payload(SRAM):
            continue
        arr_name = namify(arr.name)
        index_bits = arr.index_bits
        index_bits_type = index_bits if index_bits > 0 else 1
        metadata = dumper.array_metadata.metadata_for(arr)
        if metadata is None:
            num_write_ports = len(arr.get_write_ports())
            num_read_ports = 0
        else:
            num_write_ports = len(metadata.write_ports)
            num_read_ports = len(metadata.read_order)
        dumper.append_code(
            f'# Multi-port array {arr_name} with '
            f'{num_write_ports} write ports and {num_read_ports} read ports'
        )

        # Declare wires for write ports
        for port_idx in range(num_write_ports):
            port_suffix = f"_port{port_idx}"
            dumper.append_code(f'aw_{arr_name}_w{port_suffix} = Wire(Bits(1))')
            dumper.append_code(
                f'aw_{arr_name}_wdata{port_suffix} = Wire({dump_type(arr.scalar_ty)})'
            )
            dumper.append_code(
                f'aw_{arr_name}_widx{port_suffix} = Wire(Bits({index_bits_type}))'
            )
        # Declare wires for read ports
        for port_idx in range(num_read_ports):
            port_suffix = f"_port{port_idx}"
            if index_bits > 0:
                dumper.append_code(
                    f'aw_{arr_name}_ridx{port_suffix} = Wire(Bits({index_bits}))'
                )
            dumper.append_code(
                f'aw_{arr_name}_rdata{port_suffix} = Wire({dump_type(arr.scalar_ty)})'
            )

        # Instantiate multi-port array
        port_connections = ['clk=self.clk', 'rst=self.rst']
        for port_idx in range(num_write_ports):
            port_suffix = f"_port{port_idx}"
            port_connections.extend([
                f'w{port_suffix}=aw_{arr_name}_w{port_suffix}',
                f'wdata{port_suffix}=aw_{arr_name}_wdata{port_suffix}',
                f'widx{port_suffix}=aw_{arr_name}_widx{port_suffix}'
            ])
        for port_idx in range(num_read_ports):
            port_suffix = f"_port{port_idx}"
            if index_bits > 0:
                port_connections.append(
                    f'ridx{port_suffix}=aw_{arr_name}_ridx{port_suffix}'
                )
        dumper.append_code(
            f'array_writer_{arr_name} = {arr_name}({", ".join(port_connections)})'
        )
        for port_idx in range(num_read_ports):
            port_suffix = f"_port{port_idx}"
            dumper.append_code(
                f'aw_{arr_name}_rdata{port_suffix}.assign('
                f'array_writer_{arr_name}.rdata{port_suffix})'
            )

    # --- 2. Hardware Instantiations (Generic) ---
    dumper.append_code('\n# --- Hardware Instantiations ---')

    module_fifo_depths = {}
    all_modules = dumper.sys.modules + dumper.sys.downstreams
    default_fifo_depth = 2
    for mod in all_modules:
        module_fifo_depths[mod] = \
            {port: default_fifo_depth for port in getattr(mod, 'ports', [])}

    # Use metadata-driven pushes to compute FIFO depths, avoiding expression walking
    for module in dumper.sys.modules + dumper.sys.downstreams:
        metadata = dumper.module_metadata.get(module)
        if metadata is None:
            continue
        for push in metadata.interactions.pushes:
            fifo_port = push.fifo
            owner = fifo_port.module
            if owner not in module_fifo_depths:
                continue
            depth = push.fifo_depth
            if not isinstance(depth, int) or depth <= 0:
                depth = default_fifo_depth
            current = module_fifo_depths[owner].get(fifo_port, default_fifo_depth)
            module_fifo_depths[owner][fifo_port] = max(current, depth)

    for module in dumper.sys.modules:
        depth_map = module_fifo_depths.get(module, {})
        for port in module.ports:
            fifo_base_name = f'fifo_{namify(module.name)}_{namify(port.name)}'
            depth = depth_map.get(port, default_fifo_depth)
            dumper.append_code(
                f'{fifo_base_name}_inst = FIFO(WIDTH={port.dtype.bits}, DEPTH_LOG2={depth})'
                f'(clk=self.clk, rst_n=~self.rst, push_valid={fifo_base_name}_push_valid, '
                f'push_data={fifo_base_name}_push_data, pop_ready={fifo_base_name}_pop_ready)'
            )

            dumper.append_code(
                f'{fifo_base_name}_push_ready.assign({fifo_base_name}_inst.push_ready)'
            )
            dumper.append_code(
                f'{fifo_base_name}_pop_valid.assign({fifo_base_name}_inst.pop_valid)'
            )
            dumper.append_code(
                f'{fifo_base_name}_pop_data.assign({fifo_base_name}_inst.pop_data)'
            )

    # Instantiate TriggerCounters
    for module in dumper.sys.modules:
        tc_base_name = f'{namify(module.name)}_trigger_counter'
        dumper.append_code(
            f'{tc_base_name}_inst = TriggerCounter(WIDTH=8)'
            f'(clk=self.clk, rst_n=~self.rst, '
            f'delta={tc_base_name}_delta, pop_ready={tc_base_name}_pop_ready)'
        )
        dumper.append_code(
            f'{tc_base_name}_delta_ready.assign({tc_base_name}_inst.delta_ready)'
        )
        dumper.append_code(f'{tc_base_name}_pop_valid.assign({tc_base_name}_inst.pop_valid)')

    all_driven_fifo_ports = set()

    dumper.append_code('\n# --- Module Instantiations and Connections ---')

    all_modules = dumper.sys.modules + dumper.sys.downstreams
    downstream_order = topo_downstream_modules(dumper.sys)
    instantiation_modules = list(dumper.sys.modules) + downstream_order
    module_connection_map = {}
    pending_connection_assignments = defaultdict(list)
    declared_cross_module_wires = set()
    external_assignments_by_consumer = defaultdict(list)
    for entry in dumper.external_wire_assignments:
        external_assignments_by_consumer[entry['consumer']].append(entry)

    def _queue_cross_module_assignments(producer_module, assignments):
        target_lines = module_connection_map.get(producer_module)
        if target_lines is not None:
            target_lines.extend(assignments)
        else:
            pending_connection_assignments[producer_module].extend(assignments)

    def _declare_cross_module_wire(name, dtype_expr):
        if name not in declared_cross_module_wires:
            dumper.append_code(f'{name} = Wire({dtype_expr})')
            declared_cross_module_wires.add(name)

    def _attach_consumer_external_entries(module, port_map):
        consumer_external_entries = external_assignments_by_consumer.get(module, [])
        handled_consumer_ports = set()
        for assignment in consumer_external_entries:
            expr = assignment['expr']
            consumer_port = dumper.get_external_port_name(expr)
            if consumer_port in handled_consumer_ports:
                continue
            handled_consumer_ports.add(consumer_port)
            dtype = dump_type(expr.dtype)
            _declare_cross_module_wire(consumer_port, dtype)
            valid_name = f"{consumer_port}_valid"
            _declare_cross_module_wire(valid_name, "Bits(1)")
            port_map.append(f"{consumer_port}={consumer_port}")
            port_map.append(f"{valid_name}={valid_name}")
            producer_module = assignment['producer']
            producer_name = namify(producer_module.name)
            producer_port = dumper.external_wire_outputs.get(assignment['wire'])
            if producer_port is None:
                continue
            assignments = [
                f'{consumer_port}.assign(inst_{producer_name}.expose_{producer_port})',
                f'{valid_name}.assign(inst_{producer_name}.valid_{producer_port})',
            ]
            _queue_cross_module_assignments(producer_module, assignments)
        return handled_consumer_ports

    def _attach_external_values(module, port_map, handled_ports):
        local_ports = set()
        for ext_val in module.externals:
            if isinstance(ext_val, (Bind, ExternalIntrinsic)) or isinstance(
                    unwrap_operand(ext_val), Const):
                continue

            parent_ref = getattr(ext_val, 'parent', None)
            if isinstance(parent_ref, ModuleBase):
                producer_module = parent_ref
            else:
                producer_module = getattr(parent_ref, 'module', None)

            if producer_module is None:
                continue

            port_name = dumper.get_external_port_name(ext_val)
            if port_name in handled_ports or port_name in local_ports:
                continue

            dtype = dump_type(ext_val.dtype)
            _declare_cross_module_wire(port_name, dtype)
            valid_name = f"{port_name}_valid"
            _declare_cross_module_wire(valid_name, "Bits(1)")
            port_map.append(f"{port_name}={port_name}")
            port_map.append(f"{valid_name}={valid_name}")

            producer_name = namify(producer_module.name)
            exposed_name = dumper.dump_rval(ext_val, True, producer_name)
            assignments = [
                f'{port_name}.assign(inst_{producer_name}.expose_{exposed_name})',
                f'{valid_name}.assign(inst_{producer_name}.valid_{exposed_name})',
            ]
            _queue_cross_module_assignments(producer_module, assignments)
            local_ports.add(port_name)

        return handled_ports.union(local_ports)

    for module in instantiation_modules:  # pylint: disable=too-many-nested-blocks
        mod_name = namify(module.name)
        is_downstream = isinstance(module, Downstream)
        is_sram = isinstance(module, SRAM)

        dumper.append_code(f'# Instantiation for {module.name}')
        port_map = ['clk=self.clk', 'rst=self.rst', 'cycle_count=cycle_count']
        connection_lines = pending_connection_assignments.pop(module, [])
        module_connection_map[module] = connection_lines

        if not is_downstream:
            port_map.append(f"trigger_counter_pop_valid={mod_name}_trigger_counter_pop_valid")
            for port in module.ports:
                fifo_base_name = f'fifo_{mod_name}_{namify(port.name)}'
                if isinstance(port.dtype, Record):
                    port_map.append(f"{namify(port.name)}={fifo_base_name}_pop_data")
                else:
                    port_map.append(
                        f"{namify(port.name)}="
                        f"{fifo_base_name}_pop_data.{dump_type_cast(port.dtype)}"
                    )
                port_map.append(f"{namify(port.name)}_valid={fifo_base_name}_pop_valid")

            handled_ports = _attach_consumer_external_entries(module, port_map)
            _attach_external_values(module, port_map, handled_ports)

        else:
            upstream_modules = sorted(get_upstreams(module), key=lambda mod: mod.name)
            for dep_mod in upstream_modules:
                dep_name = namify(dep_mod.name)
                port_map.append(f"{dep_name}_executed=inst_{dep_name}.executed")

            handled_ports = _attach_consumer_external_entries(module, port_map)
            _attach_external_values(module, port_map, handled_ports)

            if is_sram:
                sram_info = get_sram_info(module)
                array = sram_info['array']
                array_name = namify(array.name)
                port_map.append(f'mem_dataout=mem_{array_name}_dataout')

        for arr in dumper.array_metadata.arrays():
            users = dumper.array_metadata.users_for(arr)
            if not any(user is module for user in users):
                continue
            # Skip SRAM arrays as they don't have array_writer instances
            if arr.is_payload(SRAM):
                continue
            read_indices = dumper.array_metadata.read_port_indices(arr, module)
            if not read_indices:
                metadata = dumper.array_metadata.metadata_for(arr)
                if metadata is not None:
                    for module_key, ports in metadata.read_ports_by_module.items():
                        if module_key is module:
                            read_indices = ports
                            break
            arr_name = namify(arr.name)
            for port_idx in read_indices:
                port_suffix = f"_port{port_idx}"
                port_map.append(
                    f"{arr_name}_rdata{port_suffix}=aw_{arr_name}_rdata{port_suffix}"
                )

        # Use metadata instead of walking expressions again
        metadata = dumper.module_metadata.get(module)
        pushes = metadata.interactions.pushes if metadata else ()
        calls = metadata.calls if metadata else []

        for push in pushes:
            # Store the actual Port object that is the target of a push
            all_driven_fifo_ports.add(push.fifo)

        unique_push_targets = {(push.fifo.module, push.fifo) for push in pushes}
        unique_call_targets = {c.bind.callee for c in calls}

        for (callee_mod, callee_port) in unique_push_targets:
            port_map.append(
                f"fifo_{namify(callee_mod.name)}_{namify(callee_port.name)}_push_ready="
                f"fifo_{namify(callee_mod.name)}_{namify(callee_port.name)}_push_ready"
            )
        for callee_mod in unique_call_targets:
            port_map.append(
                f"{namify(callee_mod.name)}_trigger_counter_delta_ready="
                f"{namify(callee_mod.name)}_trigger_counter_delta_ready"
            )

        dumper.append_code(f"inst_{mod_name} = {mod_name}({', '.join(port_map)})")

        if is_sram:
            sram_info = get_sram_info(module)
            array = sram_info['array']
            array_name = namify(array.name)
            connection_lines.extend([
                f'mem_{array_name}_address.assign(inst_{mod_name}.mem_address)',
                f'mem_{array_name}_write_data.assign(inst_{mod_name}.mem_write_data)',
                f'mem_{array_name}_write_enable.assign(inst_{mod_name}.mem_write_enable)',
                f'mem_{array_name}_read_enable.assign(inst_{mod_name}.mem_read_enable)',
            ])

        module_ports = getattr(module, 'ports', [])

        if not is_downstream:
            connection_lines.append(
                f"{mod_name}_trigger_counter_pop_ready.assign(inst_{mod_name}.executed)"
            )
            metadata = dumper.module_metadata.get(module)
            popped_fifos = {pop.fifo for pop in (metadata.interactions.pops if metadata else ())}
            for port in module_ports:
                if port in popped_fifos:
                    connection_lines.append(
                        f"fifo_{mod_name}_{namify(port.name)}_pop_ready"
                        f".assign(inst_{mod_name}.{namify(port.name)}_pop_ready)"
                    )
        else:
            for port in module_ports:
                fifo_name = f"fifo_{mod_name}_{namify(port.name)}"
                connection_lines.append(
                    f"{fifo_name}_pop_ready.assign(Bits(1)(1))"
                )

        for (callee_mod, callee_port) in unique_push_targets:
            callee_mod_name = namify(callee_mod.name)
            callee_port_name = namify(callee_port.name)
            connection_lines.append(
                f"fifo_{callee_mod_name}_{callee_port_name}_push_valid"
                f".assign(inst_{mod_name}.{callee_mod_name}_{callee_port_name}_push_valid)"
            )
            connection_lines.append(
                f"fifo_{callee_mod_name}_{callee_port_name}_push_data"
                f".assign(inst_{mod_name}.{callee_mod_name}_{callee_port_name}_push_data"
                f".as_bits())"
            )

    for module, lines in module_connection_map.items():
        if lines:
            module_connection_map[module] = list(dict.fromkeys(lines))

    has_connections = any(module_connection_map.get(m) for m in instantiation_modules)
    if has_connections:
        dumper.append_code('\n# --- Module Connections ---')
        remaining_modules = [
            module for module in instantiation_modules
            if module_connection_map.get(module)
        ]
        for idx, module in enumerate(remaining_modules):
            lines = module_connection_map[module]
            dumper.append_code(f'# Connections for {module.name}')
            for line in lines:
                dumper.append_code(line)
            if idx != len(remaining_modules) - 1:
                dumper.append_code('')
    dumper.append_code('\n# --- Global Finish Signal Collection ---')
    finish_signals = []
    for module in instantiation_modules:
        mod_name = namify(module.name)
        # Check if this module type has finish conditions using metadata
        metadata = dumper.module_metadata.get(module)
        if metadata and metadata.finish_sites:
            finish_signals.append(f'inst_{mod_name}.finish')

    if finish_signals:
        joined_signals = ", ".join(finish_signals)
        dumper.append_code(
            f'self.global_finish = reduce(operator.or_, [{joined_signals}])'
        )
    else:
        dumper.append_code('self.global_finish = Bits(1)(0)')

    # dumper.append_code('\n# --- Tie off unused FIFO push ports ---')
    for module in dumper.sys.modules:
        for port in getattr(module, 'ports', []):
            if port not in all_driven_fifo_ports:
                fifo_base_name = f'fifo_{namify(module.name)}_{namify(port.name)}'
                dumper.append_code(f'{fifo_base_name}_push_valid.assign(Bits(1)(0))')
                dumper.append_code(
                    f"{fifo_base_name}_push_data"
                    f".assign(Bits({port.dtype.bits})(0))"
                    )
    dumper.append_code('\n# --- Array Write-Back Connections ---')
    for arr_container in dumper.sys.arrays:
        owner = arr_container.owner
        if isinstance(owner, MemoryBase) and arr_container.is_payload(owner):
            continue
        metadata = dumper.array_metadata.metadata_for(arr_container)
        if metadata and metadata.users:
            dumper._connect_array(arr_container)

    dumper.append_code('\n# --- Trigger Counter Delta Connections ---')
    for module in dumper.sys.modules:
        mod_name = namify(module.name)
        async_callers = dumper.async_callers(module)
        if async_callers:
            trigger_terms = [
                f"inst_{namify(c.name)}.{mod_name}_trigger"
                for c in async_callers
            ]
            summed_triggers = f"reduce(operator.add, [{', '.join(trigger_terms)}])"

            dumper.append_code(
                f"{mod_name}_trigger_counter_delta.assign({summed_triggers}.as_bits(8))"
                )
        else:
            dumper.append_code(f"{mod_name}_trigger_counter_delta.assign(Bits(8)(1))")

    dumper.indent -= 8
    dumper.append_code('')
    dumper.append_code('system = System([Top], name="Top", output_directory="sv")')

    # Copying of external SystemVerilog files occurs during elaboration.

    dumper.append_code('system.compile()')
