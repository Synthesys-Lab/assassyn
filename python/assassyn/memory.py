from .module import Module, name_ports_of_module, implicit_fifo, combinational, wait_until
from .array import RegArray
from .block import Condition

class Memory(Module):

    def __init__(
            func,
            width,
            depth,
            latency,
            **kwargs):
        '''A decorator for marking a module with memory logic.'''
        super().__init__(**kwargs)
        assert not super().is_explicit_fifo
        self.width = width
        self.depth = depth
        self.latency = latency
        self.we = Port(Int(1))
        dtype = Int(width)
        self.addr = Port(Int(depth.bit_length()))
        self.wdata = Port(dtype)
        self.payload = RegArray(dtype, depth)
        self._attrs[Module.ATTR_MEMORY] = (width, depth, latency, self.payload)
        self.rdata = None
        self.build()
        self.wait_until()

    @wait_until
    def wait_until(self):
        '''Wait until the memory is ready.'''
        valid = None
        for port in self.ports:
            valid = port.valid() if valid is None else valid & port.valid()
        return valid

    @combinational
    def build(self):
        '''Build the memory logic.'''
        with Condition(self.we):
            self.payload[self.addr] = func.data
        self.rdata = func.payload[func.addr]
