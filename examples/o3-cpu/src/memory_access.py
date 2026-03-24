 
from assassyn.frontend import *
from opcodes import *
from scoreboard import *

class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={'index':Port(Bits(SCOREBOARD.Bit_size))},
            no_arbiter=True)
        self.name = 'm'
    @module.combinational
    def build(
        self,  
        scoreboard:Array, 
        rdata: RegArray,
    ):
        self.timing = 'systolic'
          
        index = self.index.pop()
        mdata = rdata[0].bitcast(Bits(32))
        with Condition( scoreboard['sb_status'][index] != Bits(2)(3) ):
            scoreboard['mdata'][index] = mdata
            log("mem.rdata        | 0x{:x}", mdata)
            scoreboard['sb_status'][index] = Bits(2)(3)
            
  
        return  index
