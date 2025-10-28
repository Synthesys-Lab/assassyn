"""System-level code generation utilities."""

from typing import TYPE_CHECKING, Any

from ...ir.memory.sram import SRAM
from ...ir.expr import AsyncCall, ArrayRead, ArrayWrite
from ...ir.expr.intrinsic import PureIntrinsic
from ...analysis import get_upstreams
from ..simulator.external import collect_external_intrinsics
from ...utils import unwrap_operand
from ...builder import SysBuilder
from ...utils import enforce_type
from ...ir.module.base import ModuleBase

if TYPE_CHECKING:
    from .design import CIRCTDumper
else:
    # At runtime, alias to Any to avoid cyclic import while preserving static typing
    CIRCTDumper = Any  # type: ignore

# pylint: disable=too-many-locals,too-many-branches,too-many-statements,protected-access
@enforce_type
def generate_system(dumper: CIRCTDumper, node: SysBuilder):
    """Generate code for the entire system.

    Args:
        dumper: The CIRCTDumper instance
        node: The SysBuilder instance to generate code for
    """
    sys = node
    dumper.sys = sys
    for module in sys.downstreams:
        if isinstance(module, SRAM):
            dumper.sram_payload_arrays.add(module._payload)

    external_intrinsics = collect_external_intrinsics(sys)
    dumper.external_intrinsics = external_intrinsics
    dumper.external_classes = []
    dumper.cross_module_external_reads = []
    dumper.external_outputs_by_instance.clear()
    dumper.external_output_exposures.clear()

    # Pre-populate external_instance_owners so cross-module references work
    for intrinsic in external_intrinsics:
        owner_module = intrinsic.parent
        dumper.external_instance_owners[intrinsic] = owner_module

        ext_class = intrinsic.external_class
        if ext_class not in dumper.external_classes:
            dumper.external_classes.append(ext_class)
            dumper._generate_external_module_wrapper(ext_class)

    modules_to_scan = list(sys.modules) + list(sys.downstreams)
    for module in modules_to_scan:
        body = getattr(module, "body", None)
        if body is None:
            continue
        for expr in dumper._walk_expressions(body):
            if (
                isinstance(expr, PureIntrinsic)
                and expr.opcode == PureIntrinsic.EXTERNAL_OUTPUT_READ
            ):
                instance_operand = expr.args[0]
                instance = unwrap_operand(instance_operand)
                parent_ref = getattr(instance, 'parent', None)
                if isinstance(parent_ref, ModuleBase):
                    owner_module = parent_ref
                else:
                    owner_module = getattr(parent_ref, 'module', None)
                if owner_module is None or owner_module == module:
                    continue
                port_operand = expr.args[1]
                port_name = port_operand.value if hasattr(port_operand, 'value') else port_operand
                index_operand = expr.args[2] if len(expr.args) > 2 else None
                entry = {
                    'expr': expr,
                    'producer': owner_module,
                    'consumer': module,
                    'instance': instance,
                    'port_name': port_name,
                    'index_operand': index_operand,
                }
                dumper.cross_module_external_reads.append(entry)
                dumper.external_outputs_by_instance[instance].append(entry)

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
