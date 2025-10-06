"""Array and FIFO operations code generation for Verilog.

This module contains functions to generate Verilog code for array and FIFO operations.
"""

from typing import Optional

from ....ir.expr import ArrayRead, ArrayWrite, FIFOPop, FIFOPush
from ....ir.const import Const
from ....ir.dtype import Record, Bits
from ....ir.memory.sram import SRAM
from ....utils import unwrap_operand, namify
from ..utils import dump_type, dump_type_cast


def codegen_array_read(dumper, expr: ArrayRead) -> Optional[str]:
    """Generate code for array read operations."""
    array_ref = expr.array
    is_sram_payload = False

    if isinstance(dumper.current_module, SRAM):
        if array_ref == dumper.current_module._payload:  # pylint: disable=protected-access
            is_sram_payload = True

    rval = dumper.dump_rval(expr, False)

    if is_sram_payload:
        body = f'{rval} = self.mem_dataout'
        dumper.expose('array', expr)
    else:
        array_idx = unwrap_operand(expr.idx)
        array_idx = (dumper.dump_rval(array_idx, False)
                    if not isinstance(array_idx, Const) else array_idx.value)
        index_bits = array_ref.index_bits if array_ref.index_bits > 0 else 1
        if dump_type(expr.idx.dtype) != Bits and not isinstance(array_idx, int):
            array_idx = f"{array_idx}.as_bits({index_bits})"

        array_name = dumper.dump_rval(array_ref, False)
        if isinstance(expr.dtype, Record):
            body = f'{rval} = self.{array_name}_q_in[{array_idx}]'
        else:
            body = \
            f'{rval} = self.{array_name}_q_in[{array_idx}].{dump_type_cast(expr.dtype)}'
        dumper.expose('array', expr)

    return body


def codegen_array_write(dumper, expr: ArrayWrite) -> Optional[str]:
    """Generate code for array write operations."""
    dumper.expose('array', expr)


def codegen_fifo_push(dumper, expr: FIFOPush) -> Optional[str]:
    """Generate code for FIFO push operations."""
    dumper.expose('fifo', expr)


def codegen_fifo_pop(dumper, expr: FIFOPop) -> Optional[str]:
    """Generate code for FIFO pop operations."""
    rval = namify(expr.as_operand())
    fifo_name = dumper.dump_rval(expr.fifo, False)
    dumper.expose('fifo_pop', expr)
    return f'{rval} = self.{fifo_name}'
