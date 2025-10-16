# pylint: disable=C0302
# pylint: disable=protected-access
"""Top-level harness generation for Verilog designs."""

from collections import defaultdict

from .utils import (
    dump_type,
    dump_type_cast,
    get_sram_info,
)

from ...analysis import topo_downstream_modules
from ...ir.module import Downstream
from ...ir.module.external import ExternalSV
from ...ir.memory.sram import SRAM
from ...ir.expr import (
    FIFOPush,
    FIFOPop,
    AsyncCall,
    Bind,
    Intrinsic,
    WireRead,
)
from ...ir.dtype import Record
from ...utils import namify, unwrap_operand
from ...ir.const import Const

# pylint: disable=too-many-locals,too-many-branches,too-many-statements
def generate_top_harness(dumper):
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
        if dumper.is_stub_external(module):
            continue
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
        if dumper.is_stub_external(module):
            continue
        tc_base_name = f'{namify(module.name)}_trigger_counter'
        dumper.append_code(f'# Wires for {module.name}\'s TriggerCounter')
        dumper.append_code(f'{tc_base_name}_delta = Wire(Bits(8))')
        dumper.append_code(f'{tc_base_name}_delta_ready = Wire(Bits(1))')
        dumper.append_code(f'{tc_base_name}_pop_valid = Wire(Bits(1))')
        dumper.append_code(f'{tc_base_name}_pop_ready = Wire(Bits(1))')

    for arr_container in dumper.sys.arrays:
        arr = arr_container
        is_sram_array = any(isinstance(m, SRAM) and
                           m._payload == arr for m in dumper.sys.downstreams)  # pylint: disable=protected-access
        if is_sram_array:
            continue
        arr_name = namify(arr.name)
        index_bits = arr.index_bits
        index_bits_type = index_bits if index_bits > 0 else 1
        port_mapping = dumper.array_write_port_mapping.get(arr, {})
        num_write_ports = len(port_mapping)
        read_ports = dumper.array_read_ports.get(arr, [])
        num_read_ports = len(read_ports)
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
        if index_bits > 0:
            for port_idx in range(num_read_ports):
                port_suffix = f"_port{port_idx}"
                dumper.append_code(
                    f'aw_{arr_name}_ridx{port_suffix} = Wire(Bits({index_bits}))'
                )
                dumper.append_code(
                    f'aw_{arr_name}_rdata{port_suffix} = Wire({dump_type(arr.scalar_ty)})'
                )
        else:
            for port_idx in range(num_read_ports):
                port_suffix = f"_port{port_idx}"
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
        if index_bits > 0:
            for port_idx in range(num_read_ports):
                port_suffix = f"_port{port_idx}"
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
    all_modules = [m for m in (dumper.sys.modules + dumper.sys.downstreams)
                   if not dumper.is_stub_external(m)]
    default_fifo_depth = 2
    for mod in all_modules:
        module_fifo_depths[mod] = \
            {port: default_fifo_depth for port in getattr(mod, 'ports', [])}

    for module in dumper.sys.modules + dumper.sys.downstreams:
        if not module.body:
            continue
        for expr in dumper._walk_expressions(module.body):
            if isinstance(expr, FIFOPush):
                fifo_port = expr.fifo
                owner = fifo_port.module
                if owner not in module_fifo_depths:
                    continue
                depth = getattr(expr, 'fifo_depth', None)
                if not isinstance(depth, int) or depth <= 0:
                    depth = default_fifo_depth
                current = module_fifo_depths[owner].get(fifo_port, default_fifo_depth)
                module_fifo_depths[owner][fifo_port] = max(current, depth)

    for module in dumper.sys.modules:
        if dumper.is_stub_external(module):
            continue
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
        if dumper.is_stub_external(module):
            continue
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

    def _attach_external_module_inputs(module, port_map):
        in_wires = getattr(module, 'in_wires', None)
        if not in_wires:
            return
        for _, source_expr in in_wires.items():
            if source_expr is None:
                continue
            producer_module = getattr(source_expr.parent, 'module', None)
            if producer_module is None:
                continue
            producer_name = namify(producer_module.name)
            port_name = dumper.get_external_port_name(source_expr)
            dtype = dump_type(source_expr.dtype)
            _declare_cross_module_wire(port_name, dtype)
            valid_name = f"{port_name}_valid"
            _declare_cross_module_wire(valid_name, "Bits(1)")
            port_map.append(f"{port_name}={port_name}")
            port_map.append(f"{valid_name}={valid_name}")
            exposed_name = dumper.dump_rval(source_expr, True)
            assignments = [
                f'{port_name}.assign(inst_{producer_name}.expose_{exposed_name})',
                f'{valid_name}.assign(inst_{producer_name}.valid_{exposed_name})',
            ]
            _queue_cross_module_assignments(producer_module, assignments)

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

            _attach_consumer_external_entries(module, port_map)

        else:
            if module in dumper.downstream_dependencies:
                for dep_mod in dumper.downstream_dependencies[module]:
                    dep_name = namify(dep_mod.name)
                    port_map.append(f"{dep_name}_executed=inst_{dep_name}.executed")

            _attach_consumer_external_entries(module, port_map)

            if isinstance(module, ExternalSV):
                _attach_external_module_inputs(module, port_map)
            else:
                for ext_val in module.externals:
                    if isinstance(ext_val, Bind) or isinstance(unwrap_operand(ext_val), Const):
                        continue
                    if isinstance(ext_val, WireRead):
                        wire = getattr(ext_val, 'wire', None)
                        owner = getattr(wire, 'parent', None) if wire is not None else None
                        if owner is None and wire is not None:
                            owner = getattr(wire, 'module', None)
                        if isinstance(owner, ExternalSV):
                            continue
                    producer_module = ext_val.parent.module
                    producer_ctx_name = namify(producer_module.name)
                    if isinstance(ext_val, WireRead):
                        wire = getattr(ext_val, 'wire', None)
                        owner = getattr(wire, 'parent', None) if wire is not None else None
                        if owner is None and wire is not None:
                            owner = getattr(wire, 'module', None)
                        if owner is not None:
                            producer_module = owner
                            producer_ctx_name = namify(owner.name)
                    producer_name = namify(producer_module.name)
                    port_name = dumper.get_external_port_name(ext_val)
                    exposed_name = dumper.dump_rval(ext_val, True, producer_ctx_name)
                    dtype = dump_type(ext_val.dtype)
                    _declare_cross_module_wire(port_name, dtype)
                    valid_name = f"{port_name}_valid"
                    _declare_cross_module_wire(valid_name, "Bits(1)")

                    port_map.append(f"{port_name}={port_name}")
                    port_map.append(f"{valid_name}={valid_name}")

                    assignments = [
                        f'{port_name}.assign(inst_{producer_name}.expose_{exposed_name})',
                        f'{valid_name}.assign(inst_{producer_name}.valid_{exposed_name})',
                    ]
                    _queue_cross_module_assignments(producer_module, assignments)
            if is_sram:
                sram_info = get_sram_info(module)
                array = sram_info['array']
                array_name = namify(array.name)
                port_map.append(f'mem_dataout=mem_{array_name}_dataout')

        for arr, users in dumper.array_users.items():
            if module in users:
                # Skip SRAM arrays as they don't have array_writer instances
                is_sram_array = any(isinstance(m, SRAM) and
                                   m._payload == arr for m in dumper.sys.downstreams)
                if not is_sram_array:
                    read_indices = dumper.array_read_port_mapping.get(arr, {}).get(module, [])
                    arr_name = namify(arr.name)
                    for port_idx in read_indices:
                        port_suffix = f"_port{port_idx}"
                        port_map.append(
                            f"{arr_name}_rdata{port_suffix}=aw_{arr_name}_rdata{port_suffix}"
                        )

        pushes = [e for e in dumper._walk_expressions(module.body) if isinstance(e, FIFOPush)]
        calls = [e for e in dumper._walk_expressions(module.body) if isinstance(e, AsyncCall)]

        for p in pushes:
            # Store the actual Port object that is the target of a push
            all_driven_fifo_ports.add(p.fifo)

        unique_push_targets = {(p.fifo.module, p.fifo) for p in pushes}
        unique_call_targets = {c.bind.callee for c in calls}

        # Filter out external modules from push targets
        filtered_push_targets = set()
        for (callee_mod, callee_port) in unique_push_targets:
            if not dumper.is_stub_external(callee_mod):
                filtered_push_targets.add((callee_mod, callee_port))

        # Filter out external modules from call targets
        filtered_call_targets = set()
        for callee_mod in unique_call_targets:
            if not dumper.is_stub_external(callee_mod):
                filtered_call_targets.add(callee_mod)

        for (callee_mod, callee_port) in filtered_push_targets:
            port_map.append(
                f"fifo_{namify(callee_mod.name)}_{namify(callee_port.name)}_push_ready="
                f"fifo_{namify(callee_mod.name)}_{namify(callee_port.name)}_push_ready"
            )
        for callee_mod in filtered_call_targets:
            port_map.append(
                f"{namify(callee_mod.name)}_trigger_counter_delta_ready="
                f"{namify(callee_mod.name)}_trigger_counter_delta_ready"
            )

        dumper.append_code(f"inst_{mod_name} = {mod_name}({', '.join(port_map)})")

        if isinstance(module, ExternalSV):
            continue

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
            for port in module_ports:
                if any(isinstance(e, FIFOPop) and e.fifo == port
                       for e in dumper._walk_expressions(module.body)):
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
        # Check if this module type has finish conditions
        if hasattr(module, 'body'):
            # Check if module contains FINISH intrinsics
            has_finish = any(
                isinstance(expr, Intrinsic) and expr.opcode == Intrinsic.FINISH
                for expr in dumper._walk_expressions(module.body)
            )
            if has_finish:
                finish_signals.append(f'inst_{mod_name}.finish')

    if finish_signals:
        if len(finish_signals) == 1:
            dumper.append_code(f'self.global_finish = {finish_signals[0]}')
        else:
            dumper.append_code(f'self.global_finish = {" | ".join(finish_signals)}')
    else:
        dumper.append_code('self.global_finish = Bits(1)(0)')

    # dumper.append_code('\n# --- Tie off unused FIFO push ports ---')
    for module in dumper.sys.modules:
        if dumper.is_stub_external(module):
            continue
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
        if arr_container in dumper.array_users and \
                arr_container not in dumper.sram_payload_arrays:
            dumper._connect_array(arr_container)

    dumper.append_code('\n# --- Trigger Counter Delta Connections ---')
    for module in dumper.sys.modules:
        if dumper.is_stub_external(module):
            continue
        mod_name = namify(module.name)
        if module in dumper.async_callees:
            callers_of_this_module = dumper.async_callees[module]
            trigger_terms = [
                f"inst_{namify(c.name)}.{mod_name}_trigger"
                for c in callers_of_this_module
            ]
            if len(trigger_terms) > 1:
                summed_triggers = f"({' + '.join(trigger_terms)})"
            else:
                summed_triggers = trigger_terms[0]

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
