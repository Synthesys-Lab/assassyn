"""Array and FIFO operations code generation for Verilog.

This module contains functions to generate Verilog code for array and FIFO operations.
"""

from typing import Optional

from ....ir.expr import ArrayRead, ArrayWrite, FIFOPop, FIFOPush
from ....ir.memory.sram import SRAM
from ....utils import namify


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
        array_name = dumper.dump_rval(array_ref, False)
        port_idx = dumper.array_read_expr_port.get(expr)
        if port_idx is None:
            return None
        body = f'{rval} = self.{array_name}_rdata_port{port_idx}'
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
