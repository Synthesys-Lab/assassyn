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
        dumper.expose('array', expr)
    else:
        array_name = dumper.dump_rval(array_ref, False)
        port_idx = dumper.array_metadata.read_port_index_for_expr(expr)
        if port_idx is None:
            return None
        body = f'{rval} = self.{array_name}_rdata_port{port_idx}'
        dumper.expose('array', expr)

    return body


@enforce_type
def codegen_array_write(dumper: CIRCTDumper, expr: ArrayWrite) -> Optional[str]:
    """Generate code for array write operations."""
    dumper.expose('array', expr)


@enforce_type
def codegen_fifo_push(dumper: CIRCTDumper, expr: FIFOPush) -> Optional[str]:
    """Generate code for FIFO push operations."""
    metadata = dumper.module_metadata[dumper.current_module]
    predicate = dumper.get_pred()
    fifo_metadata, entry = dumper.fifo_registry.record_push(
        dumper.current_module, expr, predicate
    )
    metadata.register_fifo_push(expr.fifo, fifo_metadata, entry)


@enforce_type
def codegen_fifo_pop(dumper: CIRCTDumper, expr: FIFOPop) -> Optional[str]:
    """Generate code for FIFO pop operations."""
    rval = namify(expr.as_operand())
    fifo_name = dumper.dump_rval(expr.fifo, False)
    metadata = dumper.module_metadata[dumper.current_module]
    predicate = dumper.get_pred()
    fifo_metadata, entry = dumper.fifo_registry.record_pop(
        dumper.current_module, expr, predicate
    )
    metadata.register_fifo_pop(expr.fifo, fifo_metadata, entry)
    return f'{rval} = self.{fifo_name}'
