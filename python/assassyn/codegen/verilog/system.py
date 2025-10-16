"""System-level code generation utilities."""

from ...ir.memory.sram import SRAM
from ...ir.expr import AsyncCall, ArrayRead, ArrayWrite
from ...analysis import get_upstreams


# pylint: disable=too-many-locals,too-many-branches,too-many-statements,protected-access
def generate_system(dumper, node):
    """Generate code for the entire system.

    Args:
        dumper: The CIRCTDumper instance
        node: The SysBuilder instance to generate code for
    """
    sys = node
    dumper.sys = sys
    for module in sys.downstreams:
        if isinstance(module, SRAM) and hasattr(module, '_payload'):
            dumper.sram_payload_arrays.add(module._payload)

    # Collect external modules
    dumper.external_modules = []
    for module in sys.modules + sys.downstreams:
        if dumper._is_external_module(module):
            if module not in dumper.external_modules:
                dumper.external_modules.append(module)
        # Also check for external modules used within downstream modules
        for expr in dumper._walk_expressions(module.body):
            if isinstance(expr, AsyncCall):
                callee = expr.bind.callee
                if dumper._is_external_module(callee):
                    if callee not in dumper.external_modules:
                        dumper.external_modules.append(callee)

    # Generate PyCDE wrapper classes for external modules first
    for ext_module in dumper.external_modules:
        dumper._generate_external_module_wrapper(ext_module)

    for arr_container in sys.arrays:
        if arr_container in dumper.sram_payload_arrays:
            continue
        sub_array = arr_container
        if sub_array not in dumper.array_write_port_mapping:
            dumper.array_write_port_mapping[sub_array] = {}
        sub_array_writers = sub_array.get_write_ports()
        for module, _ in sub_array_writers.items():
            if module not in dumper.array_write_port_mapping[sub_array]:
                port_idx = len(dumper.array_write_port_mapping[sub_array])
                dumper.array_write_port_mapping[sub_array][module] = port_idx

        # Record read port usage per array and module.
        arr_reads = []
        for module in sys.modules + sys.downstreams:
            if module.body is None:
                continue
            reads = [
                expr for expr in dumper._walk_expressions(module.body)
                if isinstance(expr, ArrayRead) and expr.array == sub_array
            ]
            if reads:
                arr_reads.append((module, reads))

        dumper.array_read_port_mapping[sub_array] = {}
        dumper.array_read_ports[sub_array] = []
        port_counter = 0
        for module, reads in arr_reads:
            dumper.array_read_port_mapping[sub_array][module] = []
            for read_expr in reads:
                dumper.array_read_port_mapping[sub_array][module].append(port_counter)
                dumper.array_read_ports[sub_array].append((module, read_expr))
                dumper.array_read_expr_port[read_expr] = port_counter
                port_counter += 1

    for arr_container in sys.arrays:
        if arr_container not in dumper.sram_payload_arrays:
            dumper.visit_array(arr_container)

    expr_to_module = {}
    for module in sys.modules + sys.downstreams:
        for expr in dumper._walk_expressions(module.body):
            if expr.is_valued():
                expr_to_module[expr] = module

    for ds_module in sys.downstreams:
        dumper.downstream_dependencies[ds_module] = get_upstreams(ds_module)

    all_modules = dumper.sys.modules + dumper.sys.downstreams
    for module in all_modules:
        for expr in dumper._walk_expressions(module.body):
            if isinstance(expr, AsyncCall):
                callee = expr.bind.callee
                if callee not in dumper.async_callees:
                    dumper.async_callees[callee] = []

                if module not in dumper.async_callees[callee]:
                    dumper.async_callees[callee].append(module)

    dumper.array_users = {}
    # pylint: disable=R1702
    for arr_container in dumper.sys.arrays:
        if arr_container in dumper.sram_payload_arrays:
            continue
        arr = arr_container
        dumper.array_users[arr] = []
        for mod in dumper.sys.modules + dumper.sys.downstreams:
            if isinstance(mod, SRAM) and hasattr(mod, 'payload') and arr == mod.payload:
                continue
            for expr in dumper._walk_expressions(mod.body):
                if isinstance(expr, (ArrayRead, ArrayWrite)) and expr.array == arr:
                    if mod not in dumper.array_users[arr]:
                        dumper.array_users[arr].append(mod)

    # Process only non-external modules from sys.modules
    for elem in sys.modules:
        if dumper.is_stub_external(elem):
            continue

        dumper.current_module = elem
        dumper.visit_module(elem)
    dumper.current_module = None
    for elem in sys.downstreams:
        if dumper.is_stub_external(elem):
            continue
        dumper.current_module = elem
        dumper.visit_module(elem)
    dumper.current_module = None
    dumper.is_top_generation = True
    # Import here to avoid circular dependency
    from .top import generate_top_harness  # pylint: disable=import-outside-toplevel
    generate_top_harness(dumper)
    dumper.is_top_generation = False
