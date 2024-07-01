from .module import Module, name_ports_of_module, implicit_fifo
from .array import RegArray
from .block import Condition

@decorator
def constructor(func, depth=None, width=None, latency=(1, 1), *args, **kwargs):
    assert width is not None, "width must be specified"
    assert width & -width == width, "width must be a power of 2"
    assert depth is not None, "depth must be specified"
    assert depth & -depth == depth, "depth must be a power of 2"
    latency = kwargs.get('latency', None)
    super(type(module_self), module_self).__init__(width, depth, latency, attrs)
    name_ports_of_module(module_self)


class Memory(Module):

    def __init__(func, width, depth, latency, attrs):
        '''A decorator for marking a module with memory logic.'''
        super().__init__(attrs)
        self.width = width
        self.depth = depth
        self.latency = latency
        self.we = Port(Int(1))
        dtype = Int(width)
        self.addr = Port(Int(depth.bit_length()))
        self.wdata = Port(dtype)
        self.attrs.add(Module.ATTR_MEMORY)
        self.payload = RegArray(dtype, depth)
        self.rdata = None

    @combinational
    def build(self):
        '''Build the memory logic.'''
        with Condition(self.we):
            func.payload[func.addr] = func.data
        self.rdata = func.payload[func.addr]
