"""Runtime PyCDE helpers shared between generated and hand-authored designs."""
# pylint: disable=invalid-name,unused-argument

from pycde import Clock, Input, Module, Output
from pycde import modparams
from pycde.types import Bits

__all__ = ("FIFO", "TriggerCounter")


@modparams
def FIFO(WIDTH: int, DEPTH_LOG2: int):
    """Depth-parameterized FIFO matching the backend's SystemVerilog resource."""

    class FIFOImpl(Module):
        """PyCDE module for the backend FIFO primitive."""
        module_name = "fifo"
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
    """Credit counter primitive used to gate driver execution."""

    class TriggerCounterImpl(Module):
        """PyCDE module mirroring the trigger_counter primitive."""
        module_name = "trigger_counter"
        clk = Clock()
        rst_n = Input(Bits(1))
        delta = Input(Bits(WIDTH))
        delta_ready = Output(Bits(1))
        pop_ready = Input(Bits(1))
        pop_valid = Output(Bits(1))

    return TriggerCounterImpl
