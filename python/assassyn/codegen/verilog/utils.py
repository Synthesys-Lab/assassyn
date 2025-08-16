"""Utility functions for the Verilog backend."""

from typing import Optional

from ...ir.module import Module
from ...ir.expr import Intrinsic
from ...ir.dtype import Int, UInt, Bits, DType, Record

def find_wait_until(module: Module) -> Optional[Intrinsic]:
    """Find the WAIT_UNTIL intrinsic in a module if it exists."""
    for elem in module.body.body:
        if isinstance(elem, Intrinsic):
            if elem.opcode == Intrinsic.WAIT_UNTIL:
                return elem
    return None


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

@modparams
def FIFO(WIDTH: int, DEPTH_LOG2: int):
    class FIFOImpl(Module):
        module_name = f"fifo"
        # Define inputs
        clk = Clock()
        rst_n = Input(Bits(1))
        push_valid = Input(Bits(1))
        push_data = Input(Bits(WIDTH))
        pop_ready = Input(Bits(1))
        # Define outputs
        push_ready = Output(Bits(1))
        pop_valid = Output(Bits(1))
        pop_data = Output(Bits(WIDTH))
    return FIFOImpl


@modparams
def TriggerCounter(WIDTH: int):
    class TriggerCounterImpl(Module):
        module_name = f"trigger_counter"
        clk = Clock()
        rst_n = Input(Bits(1))
        delta = Input(Bits(WIDTH))
        delta_ready = Output(Bits(1))
        pop_ready = Input(Bits(1))
        pop_valid = Output(Bits(1))
    return TriggerCounterImpl

'''