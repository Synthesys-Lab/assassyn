"""System-level code generation utilities."""

from typing import TYPE_CHECKING, Any

from ...ir.memory.base import MemoryBase
from ...builder import SysBuilder
from ...utils import enforce_type

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

    dumper.external_output_exposures.clear()
    dumper.external_wire_assignments.clear()
    dumper.external_wire_assignment_keys.clear()
    dumper.external_wire_outputs.clear()
    dumper.external_instance_names.clear()
    dumper.external_wrapper_names.clear()

    for ext_class in dumper.external_metadata.classes:
        dumper._generate_external_module_wrapper(ext_class)

    dumper.array_metadata.collect(sys)

    for arr_container in sys.arrays:
        owner = arr_container.owner
        if isinstance(owner, MemoryBase) and arr_container.is_payload(owner):
            continue
        dumper.visit_array(arr_container)

    # Process every module from sys.modules
    for elem in sys.modules:
        dumper.current_module = elem
        dumper.visit_module(elem)
    dumper.current_module = None
    for elem in sys.downstreams:
        dumper.current_module = elem
        dumper.visit_module(elem)
    dumper.current_module = None
    dumper.is_top_generation = True
    # Import here to avoid circular dependency
    from .top import generate_top_harness  # pylint: disable=import-outside-toplevel
    generate_top_harness(dumper)
    dumper.is_top_generation = False
