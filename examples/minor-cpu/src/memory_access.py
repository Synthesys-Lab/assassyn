 
from assassyn.frontend import *
from opcodes import *
class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={'rdata': Port(Bits(32)), 'rd': Port(Bits(5))},
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
            rd = self.rd.pop()
            log("mem.rdata        | 0x{:x}", data)
            mem_bypass_reg[0] = rd
            with Condition(rd != Bits(5)(0)):
                log("mem.bypass       | x{:02} = 0x{:x}", rd, data)
                mem_bypass_data[0] = data

        with Condition(~self.rdata.valid()):
            mem_bypass_reg[0] = Bits(5)(0)

        arg = self.rdata.valid().select(self.rdata.peek(), Bits(32)(0))

        wb_bound = writeback.bind(mdata = arg)
        wb_bound.async_called() 
        wb_bound.set_fifo_depth(mdata = 2)
