"""Array read/write code generation helpers for simulator."""

# pylint: disable=unused-argument

from ....utils import namify
from ..node_dumper import dump_rval_ref
from ..port_mapper import get_port_manager


def codegen_array_read(node, module_ctx):
    """Generate code for array read operations."""
    array = node.array
    idx = node.idx
    array_name = namify(array.name)
    idx_val = dump_rval_ref(module_ctx, idx)
    return f"sim.{array_name}.payload[{idx_val} as usize].clone()"


def codegen_array_write(node, module_ctx, module_name):
    """Generate code for array write operations with port indexing."""
    array = node.array
    idx = node.idx
    value = node.val
    module = node.module

    array_name = namify(array.name)
    idx_val = dump_rval_ref(module_ctx, idx)
    value_val = dump_rval_ref(module_ctx, value)
    module_writer = namify(module.name)

    manager = get_port_manager()
    port_idx = manager.get_or_assign_port(array_name, module_writer)

    return f"""{{
              let stamp = sim.stamp - sim.stamp % 100 + 50;
              let write = ArrayWrite::new(stamp, {idx_val} as usize,
                                         {value_val}.clone(), "{module_writer}");
              sim.{array_name}.write({port_idx}, write);
            }}"""
