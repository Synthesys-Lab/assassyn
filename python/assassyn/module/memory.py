'''Memory module, a special and subclass of Module.'''

from .module import Module
from ..dtype import Bits
from ..array import RegArray
from ..block import Condition


class SRAM:
    '''The SRAM type of instantiating an SRAM logic.'''

    #pylint: disable=too-many-arguments, too-few-public-methods
    def __init__(self, width, depth, init_file, we, re, addr, wdata, user: Module):
        '''The constructor for the SRAM module.
        # Arguments
        width: int: The width of the data.
        depth: int: The depth of the memory.
        init_file: str: The file to initialize the memory.
        we: Value: The write enable signal.
        re: Value: The read enable signal.
        addr: Value: The address signal.
        wdata: Value: The write data signal.
        user: Module: The user module, it is required to have a rdata port.
        '''
        self.width = width
        self.depth = depth
        self.init_file = init_file
        self.payload = RegArray(Bits(width), depth)
        # TODO(@were): Put this into the block box scope.
        with Condition(we):
            self.payload[addr] = wdata
        with Condition(re):
            user.bind(rdata=self.payload[addr])
        self.bound = user
