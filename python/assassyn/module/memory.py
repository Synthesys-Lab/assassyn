'''Memory module, a special and subclass of Module.'''

from .module import Module, Port
from ..dtype import Int, Bits
from ..array import RegArray
from ..block import Condition

# pylint: disable=too-many-instance-attributes
class Memory(Module):
    '''The memory class, a subclass of Module.
    This class is aimed at emulate an external module with memory behavior.'''

    def __init__(
            self,
            width,
            depth,
            latency=(1, 1),
            init_file=None,
            **kwargs):
        '''A decorator for marking a module with memory logic.'''
        super().__init__(**kwargs)
        assert not super().is_explicit_fifo
        self.width = width
        self.depth = depth
        self.latency = latency
        self.init_file = init_file
        self.we = Port(Int(1))
        dtype = Bits(width)
        self.addr = Port(Int(max(1, depth.bit_length() - 1)))
        self.wdata = Port(dtype)
        self.payload = RegArray(dtype, depth)
        self._attrs[Module.ATTR_MEMORY] = (width, depth, latency, self.payload)
        self.rdata = None

    def build(self):
        '''Build the memory logic.'''
        with Condition(self.we):
            self.payload[self.addr] = self.wdata
        self.rdata = self.payload[self.addr]
