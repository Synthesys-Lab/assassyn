import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Decoder(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self):
        pass


class Fetcher(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, decoder: Memory, pc: Array, on_branch: Array):
        with Condition(~on_branch[0]):
            log("Fetching instruction at PC: 0x{:x}", pc[0])
            to_fetch = pc[0][2:11].bitcast(UInt(10))
            decoder.async_called(write = Bits(1)(0), wdata = Bits(32)(0), addr = to_fetch)
            pc[0] = (pc[0].bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))

        with Condition(on_branch[0]):
            log("on a branch, stall fetching, pc freeze at 0x{:x}", pc[0])


class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, fetcher: Module):
        fetcher.async_called()