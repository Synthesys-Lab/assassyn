"""Array and FIFO operations code generation for Verilog.

This module contains functions to generate Verilog code for array and FIFO operations.
"""

from __future__ import annotations

from typing import Optional, TYPE_CHECKING

from ....ir.expr import ArrayRead, ArrayWrite, FIFOPop, FIFOPush
from ....ir.memory.sram import SRAM
from ....utils import namify
from ....utils.enforce_type import enforce_type

if TYPE_CHECKING:
    from ..design import CIRCTDumper


@enforce_type
def codegen_array_read(dumper: CIRCTDumper, expr: ArrayRead) -> Optional[str]:
    """Generate code for array read operations."""
    array_ref = expr.array
    is_sram_payload = False

    if isinstance(dumper.current_module, SRAM):
        if array_ref.is_payload(dumper.current_module):
            is_sram_payload = True

    rval = dumper.dump_rval(expr, False)

    if is_sram_payload:
        body = f'{rval} = self.mem_dataout'
    else:
        array_name = dumper.dump_rval(array_ref, False)
        port_idx = dumper.array_metadata.read_port_index_for_expr(expr)
        if port_idx is None:
            return None
        body = f'{rval} = self.{array_name}_rdata_port{port_idx}'

    return body


@enforce_type
def codegen_array_write(_dumper: CIRCTDumper, _expr: ArrayWrite) -> Optional[str]:
    """Generate code for array write operations."""
    return None

@enforce_type
def codegen_fifo_push(_dumper: CIRCTDumper, _expr: FIFOPush) -> Optional[str]:
    """Generate code for FIFO push operations."""
    # FIFO interactions are recorded during the analysis pre-pass.
    return None


@enforce_type
def codegen_fifo_pop(dumper: CIRCTDumper, expr: FIFOPop) -> Optional[str]:
    """Generate code for FIFO pop operations."""
    rval = namify(expr.as_operand())
    fifo_name = dumper.dump_rval(expr.fifo, False)
    return f'{rval} = self.{fifo_name}'
