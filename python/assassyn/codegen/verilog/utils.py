"""Utility functions for the Verilog backend."""
import re
from typing import Optional

from ...ir.module import Module
from ...ir.memory.sram import SRAM
from ...ir.expr import Intrinsic
from ...ir.dtype import Int, UInt, Bits, DType, Record
from ...utils import namify

def get_sram_info(node: SRAM) -> dict:
    """Extract SRAM-specific information."""
    return {  # pylint: disable=protected-access
        'array': node._payload,
        'init_file': node.init_file,
        'width': node.width,
        'depth': node.depth
    }


def extract_sram_params(node: SRAM) -> dict:
    """Extract common SRAM parameters from an SRAM module.

    Args:
        sram: SRAM module object

    Returns:
        dict: Dictionary containing array_name, data_width, and addr_width
    """
    sram_info = get_sram_info(node)
    array = sram_info['array']
    array_name = namify(array.name)
    data_width = array.scalar_ty.bits
    addr_width = array.index_bits if array.index_bits > 0 else 1

    return {
        'sram_info': sram_info,
        'array': array,
        'array_name': array_name,
        'data_width': data_width,
        'addr_width': addr_width
    }

def find_wait_until(module: Module) -> Optional[Intrinsic]:
    """Find the WAIT_UNTIL intrinsic in a module if it exists."""
    body = getattr(module, 'body', None) or []
    for elem in body:
        if isinstance(elem, Intrinsic):
            if elem.opcode == Intrinsic.WAIT_UNTIL:
                return elem
    return None


def ensure_bits(expr_str: str) -> str:
    """Ensure an expression is of Bits type, converting if necessary."""
    uint_pattern = r'UInt\(([^)]+)\)\(([^)]+)\)'
    if re.search(uint_pattern, expr_str):
        expr_str = re.sub(uint_pattern, r'Bits(\1)(\2)', expr_str)
        return expr_str
    if "Bits(" in expr_str:
        return expr_str
    if ".as_bits()" in expr_str:
        return expr_str
    if any(pattern in expr_str for pattern in \
           ["executed_wire", "_valid", "_pop_valid", "_push_valid"]):
        return expr_str
    return f"{expr_str}.as_bits()"



def dump_type(ty: DType) -> str:
    """Dump a type to a string."""

    if isinstance(ty, Int):
        return f"SInt({ty.bits})"
    if isinstance(ty, UInt):
        return f"UInt({ty.bits})"
    if isinstance(ty, Bits):
        return f"Bits({ty.bits})"
    if isinstance(ty, Record):
        return f"Bits({ty.bits})"

    if isinstance(ty, slice):
        width = ty.stop - ty.start + 1
        return f"Bits({width})"
    raise ValueError(f"Unknown type: {type(ty)}")

def dump_type_cast(ty: DType,bits:int = None) -> str:
    """Dump a type to a string."""
    if isinstance(ty, Int):
        name = "sint"
    elif isinstance(ty, UInt):
        name = "uint"
    elif isinstance(ty, (Bits, Record)):
        name = "bits"
    else:
        raise ValueError(f"Unknown type: {type(ty)}")
    value = bits
    if value is None and hasattr(ty, 'bits'):
        value = ty.bits

    return f"as_{name}({value})"

HEADER = '''from pycde import Input, Output, Module, System, Clock, Reset,dim
from pycde import generator, modparams
from pycde.constructs import Reg, Array, Mux,Wire
from pycde.types import Bits, SInt, UInt
from pycde.signals import Struct, BitsSignal
from pycde.dialects import comb,sv
from functools import reduce
from operator import or_, and_, add
from assassyn.pycde_wrapper import FIFO, TriggerCounter, build_register_file

'''
