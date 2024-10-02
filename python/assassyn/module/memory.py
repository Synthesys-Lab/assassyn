'''Memory module, a special and subclass of Module.'''

from .module import Module
from ..block import SRAMBox
from ..dtype import Bits
from ..array import RegArray
from ..block import Condition


def SRAM(width, depth, init_file, we, re, addr, wdata, user: Module):
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

    # Returns
    bound: Bind: The bound handle of the user module.
    '''
    with SRAMBox(width, depth, init_file, we, re, addr, wdata) as sram:
        payload = RegArray(Bits(width), depth, attr=[sram])
        # TODO(@were): Put this into the block box scope.
        with Condition(we):
            self.payload[addr] = wdata
        with Condition(re):
            bound = user.bind(rdata=self.payload[addr])
    return bound
