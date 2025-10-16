"""Module port generation utilities for Verilog code generation."""

from typing import List
from .utils import dump_type, get_sram_info
from ...ir.module import Module
from ...ir.expr import FIFOPop, Bind
from ...ir.const import Const
from ...utils import namify, unwrap_operand


# pylint: disable=too-many-arguments,too-many-locals,too-many-branches,too-many-statements
# pylint: disable=protected-access
def generate_module_ports(dumper, node: Module, is_downstream: bool, is_sram: bool,
                          is_driver: bool, pushes: List, calls: List) -> None:
    """Generate port declarations for a module.

    Args:
        dumper: The CIRCTDumper instance
        node: The module to generate ports for
        is_downstream: Whether this is a downstream module
        is_sram: Whether this is an SRAM module
        is_driver: Whether this module is a driver
        pushes: List of FIFOPush expressions
        calls: List of AsyncCall expressions
    """
    dumper.append_code('clk = Clock()')
    dumper.append_code('rst = Reset()')
    dumper.append_code('executed = Output(Bits(1))')
    dumper.append_code('cycle_count = Input(UInt(64))')
    dumper.append_code('finish = Output(Bits(1))')

    if is_downstream:
        if node in dumper.downstream_dependencies:
            for dep_mod in dumper.downstream_dependencies[node]:
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

    elif is_driver or node in dumper.async_callees:
        dumper.append_code('trigger_counter_pop_valid = Input(Bits(1))')

    for ext_val in node.externals:
        if isinstance(ext_val, Bind) or isinstance(unwrap_operand(ext_val), Const):
            continue
        port_name = dumper.get_external_port_name(ext_val)
        port_type = dump_type(ext_val.dtype)
        dumper.append_code(f'{port_name} = Input({port_type})')
        dumper.append_code(f'{port_name}_valid = Input(Bits(1))')

    if not is_downstream and not dumper._is_external_module(node):
        for i in node.ports:
            name = namify(i.name)
            dumper.append_code(f'{name} = Input({dump_type(i.dtype)})')
            dumper.append_code(f'{name}_valid = Input(Bits(1))')
            has_pop = any(
                isinstance(e, FIFOPop) and e.fifo == i
                for e in dumper._walk_expressions(node.body)
            )
            if has_pop:
                dumper.append_code(f'{name}_pop_ready = Output(Bits(1))')

    unique_push_handshake_targets = {(p.fifo.module, p.fifo.name) for p in pushes}
    unique_call_handshake_targets = {c.bind.callee for c in calls}
    unique_output_push_ports = {p.fifo for p in pushes}

    # Skip external modules for handshake targets
    filtered_push_targets = set()
    for module, fifo_name in unique_push_handshake_targets:
        if not dumper._is_external_module(module):
            filtered_push_targets.add((module, fifo_name))

    filtered_call_targets = set()
    for callee in unique_call_handshake_targets:
        if not dumper._is_external_module(callee):
            filtered_call_targets.add(callee)

    for module, fifo_name in filtered_push_targets:
        port_name = f'fifo_{namify(module.name)}_{namify(fifo_name)}_push_ready'
        dumper.append_code(f'{port_name} = Input(Bits(1))')
    for callee in filtered_call_targets:
        port_name = f'{namify(callee.name)}_trigger_counter_delta_ready'
        dumper.append_code(f'{port_name} = Input(Bits(1))')

    # Skip external modules for output push ports
    filtered_output_push_ports = set()
    for fifo_port in unique_output_push_ports:
        if not dumper._is_external_module(fifo_port.module):
            filtered_output_push_ports.add(fifo_port)

    for fifo_port in filtered_output_push_ports:
        port_prefix = f"{namify(fifo_port.module.name)}_{namify(fifo_port.name)}"
        dumper.append_code(f'{port_prefix}_push_valid = Output(Bits(1))')
        dtype = fifo_port.dtype
        dumper.append_code(f'{port_prefix}_push_data = Output({dump_type(dtype)})')
    for callee in filtered_call_targets:
        dumper.append_code(f'{namify(callee.name)}_trigger = Output(UInt(8))')

    # pylint: disable=too-many-nested-blocks
    for arr_container in dumper.sys.arrays:
        arr = arr_container
        if is_sram:
            sram_info = get_sram_info(node)
            if sram_info and arr == sram_info['array']:
                continue
        read_mapping = dumper.array_read_port_mapping.get(arr, {})
        read_port_indices = read_mapping.get(node)
        if read_port_indices is None:
            for module_key, ports in read_mapping.items():
                if module_key is node:
                    read_port_indices = ports
                    break
        if read_port_indices is None:
            read_port_indices = []

        port_mapping = dumper.array_write_port_mapping.get(arr, {})
        writes_idx = port_mapping.get(node)
        if writes_idx is None:
            for module_key, idx in port_mapping.items():
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

    for port_code in dumper.exposed_ports_to_add:
        dumper.append_code(port_code)
