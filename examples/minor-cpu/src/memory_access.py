 
from assassyn.frontend import *
from opcodes import *
class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={'rdata': Port(Bits(32)), 'rd': Port(Bits(5)),'mem_ext' : Port(Bits(2)),'result': Port(Bits(32))},
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

        data_cut = Bits(32)(0)
        mem_ext = self.mem_ext.pop()
        result = self.result.pop()
        rd = self.rd.pop()
        with Condition(self.rdata.valid()):
            data = self.rdata.pop()
            log("mem.rdata        | 0x{:x}", data)
            mem_bypass_reg[0] = rd
            with Condition(rd != Bits(5)(0)):
                log("mem.bypass       | x{:02} = 0x{:x}", rd, data)
                mem_bypass_data[0] = data

        with Condition(~self.rdata.valid()):
            mem_bypass_reg[0] = Bits(5)(0)

        arg = self.rdata.valid().select(self.rdata.peek(), Bits(32)(0))
        sign = arg[7:7]
        ext = sign.select(Bits(24)(0xffffff), Bits(24)(0))
        data_cut = mem_ext[1:1].select( Bits(24)(0).concat(arg[0:7]) , ext.concat(arg[0:7]) )
        
        arg = self.rdata.valid().select(arg, result)
        arg = mem_ext[0:0].select( data_cut ,arg)

        wb_bound = writeback.bind(mdata = arg , rd = rd)
        wb_bound.async_called() 
