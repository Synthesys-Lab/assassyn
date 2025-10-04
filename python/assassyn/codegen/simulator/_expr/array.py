"""Array read/write code generation helpers for simulator."""

# pylint: disable=unused-argument

from ....utils import namify
from ..node_dumper import dump_rval_ref


def codegen_array_read(node, module_ctx, sys):
    """Generate code for array read operations."""
    array = node.array
    idx = node.idx
    array_name = namify(array.name)
    idx_val = dump_rval_ref(module_ctx, sys, idx)
    return f"sim.{array_name}.payload[{idx_val} as usize].clone()"


def codegen_array_write(node, module_ctx, sys, module_name):
    """Generate code for array write operations."""
    array = node.array
    idx = node.idx
    value = node.val
    module = node.module

    array_name = namify(array.name)
    idx_val = dump_rval_ref(module_ctx, sys, idx)
    value_val = dump_rval_ref(module_ctx, sys, value)
    module_writer = namify(module.name)
    port_id = id(module)  # Use module id as port identifier

    return f"""{{
              let stamp = sim.stamp - sim.stamp % 100 + 50;
              sim.{array_name}.write_port.push(
                ArrayWrite::new(stamp, {idx_val} as usize, \
                      {value_val}.clone(), "{module_writer}", {port_id}));
            }}"""
