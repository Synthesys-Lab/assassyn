 
from assassyn.frontend import *
from opcodes import *
from scoreboard import *

class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={'rdata': Port(Bits(32)),  
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
          
        index = self.index.pop() 
        with Condition( scoreboard['sb_status'][index] != Bits(2)(3) ):
            with Condition(self.rdata.valid()):
                mdata = self.rdata.pop()
                
                log("mem.rdata        | 0x{:x}", mdata) 
                  
            arg = self.rdata.valid().select(self.rdata.peek(),Bits(32)(0))
                   
        writeback.async_called()
        
        return  mdata,arg,index