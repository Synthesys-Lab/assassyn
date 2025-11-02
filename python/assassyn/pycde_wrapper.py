"""Runtime PyCDE helpers shared between generated and hand-authored designs."""

from __future__ import annotations
# pylint: disable=invalid-name,unused-argument,import-error,too-few-public-methods

from pycde import Clock, Input, Module, Output, Reset, dim, generator
from pycde import modparams
from pycde.constructs import Mux, Reg
from pycde.types import Bits

__all__ = ("FIFO", "TriggerCounter", "build_register_file")


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


def build_register_file(  # pylint: disable=too-many-arguments
    module_name,
    data_type,
    depth,
    num_write_ports,
    num_read_ports,
    *,
    addr_width=None,
    include_read_index=True,
    initializer=None,
):
    """Create a parameterized register file module with the requested port counts."""
    computed_addr_width = max(1, (depth - 1).bit_length()) if depth > 0 else 1
    if addr_width is None:
        addr_width = computed_addr_width
    addr_width = max(1, addr_width)

    initializer_values = None
    if initializer is not None:
        if len(initializer) != depth:
            raise ValueError(
                f"Initializer length {len(initializer)} does not match depth {depth}"
            )
        initializer_values = list(initializer)

    attrs = {
        "module_name": module_name,
        "clk": Clock(),
        "rst": Reset(),
        "ADDR_WIDTH": addr_width,
        "DEPTH": depth,
        "NUM_WRITE_PORTS": num_write_ports,
        "NUM_READ_PORTS": num_read_ports,
    }

    for w_idx in range(num_write_ports):
        attrs[f"w_port{w_idx}"] = Input(Bits(1))
        attrs[f"widx_port{w_idx}"] = Input(Bits(addr_width))
        attrs[f"wdata_port{w_idx}"] = Input(data_type)

    if include_read_index:
        for r_idx in range(num_read_ports):
            attrs[f"ridx_port{r_idx}"] = Input(Bits(addr_width))

    for r_idx in range(num_read_ports):
        attrs[f"rdata_port{r_idx}"] = Output(data_type)

    @generator
    def construct(self):  # pylint: disable=too-many-locals
        if initializer_values is None:
            reset_literal = [data_type(0) for _ in range(depth)]
        else:
            reset_literal = []
            for value in initializer_values:
                if hasattr(value, "dtype"):
                    reset_literal.append(value)
                else:
                    reset_literal.append(data_type(value))

        data_reg = Reg(
            dim(data_type, depth),
            clk=self.clk,
            rst=self.rst,
            rst_value=reset_literal,
        )

        index_literals = [Bits(addr_width)(i) for i in range(depth)]
        next_data_values = []

        for element_idx, current_literal in enumerate(index_literals):
            element_value = data_reg[element_idx]
            for port_idx in reversed(range(num_write_ports)):
                write_en = getattr(self, f"w_port{port_idx}")
                write_idx = getattr(self, f"widx_port{port_idx}")
                write_data = getattr(self, f"wdata_port{port_idx}")
                match = (write_idx == current_literal).as_bits(1)
                element_value = Mux(write_en & match, element_value, write_data)
            next_data_values.append(element_value)

        data_reg.assign(dim(data_type, depth)(next_data_values))

        for port_idx in range(num_read_ports):
            read_value = data_reg[0]
            if include_read_index:
                read_idx = getattr(self, f"ridx_port{port_idx}")
                for element_idx, current_literal in enumerate(index_literals[1:], start=1):
                    match = (read_idx == current_literal).as_bits(1)
                    read_value = Mux(match, read_value, data_reg[element_idx])
            setattr(self, f"rdata_port{port_idx}", read_value)

    attrs["construct"] = construct
    return type(module_name, (Module,), attrs)
