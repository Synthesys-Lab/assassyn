 
from assassyn.frontend import *
from opcodes import *
class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={'rdata': Port(Bits(32))},
            no_arbiter=True)
        self.name = 'm'

    @module.combinational
    def build(
        self, 
        writeback: Module, 
        mem_bypass_reg: Array, 
        mem_bypass_data: Array
    ):
        self.timing = 'systolic'

        with Condition(self.rdata.valid()):
            data = self.rdata.pop()
            log("mem.rdata        | 0x{:x}", data)
            with Condition(mem_bypass_reg[0] != Bits(5)(0)):
                log("mem.bypass       | x{:02} = 0x{:x}", mem_bypass_reg[0], data)
            mem_bypass_data[0] = (mem_bypass_reg[0] != Bits(5)(0)).select(data, Bits(32)(0))

        arg = self.rdata.valid().select(self.rdata.peek(), Bits(32)(0))
        writeback.async_called(mdata = arg)
