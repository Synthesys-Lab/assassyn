from .module import Module

class Memory(Module):

    def __init__(func, width, depth, latency):
        '''A decorator for marking a module with memory logic.'''
        super().__init__()
        self.width = width
        self.depth = depth
        self.latency = latency
        self.we = Port(Int(1))
        self.addr = Port(Int(depth.bit_length()))
        self.data = Port(Int(width))
        self.attrs.add(Module.ATTR_MEMORY)

