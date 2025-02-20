 
from assassyn.frontend import *
from opcodes import *
from scoreboard import *

class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={'rdata': Port(Bits(32)), 
            'rd': Port(Bits(5)),
            'index':Port(Bits(SCOREBOARD.Bit_size))},
            no_arbiter=True)
        self.name = 'm'
    @module.combinational
    def build(
        self, 
        writeback: Module,
        scoreboard:Array,
        
    ):
        self.timing = 'systolic'
        
        log("in mem")
        rd = self.rd.pop()
        index = self.index.pop()
        mem_update = NoDep
        with Condition( scoreboard['sb_status'][index] != Bits(2)(3) ):
            with Condition(self.rdata.valid()):
                data = self.rdata.pop()
                
                log("mem.rdata        | 0x{:x}", data)
                
                with Condition(rd != Bits(5)(0)):
                    log("mem.bypass       | x{:02} = 0x{:x}", rd, data)
                    mem_update = index
                    mdata = data
             
            arg = self.rdata.valid().select(self.rdata.peek(),Bits(32)(0))
                   
        writeback.async_called()
        
        return mem_update,mdata,arg,index