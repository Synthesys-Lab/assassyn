"""Utility functions for the Verilog backend."""

from typing import Optional

from ...ir.module import Module
from ...ir.expr import Intrinsic

def find_wait_until(module: Module) -> Optional[Intrinsic]:
    """Find the WAIT_UNTIL intrinsic in a module if it exists."""
    for elem in module.body.body:
        if isinstance(elem, Intrinsic):
            if elem.opcode == Intrinsic.WAIT_UNTIL:
                return elem
    return None
