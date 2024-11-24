 
from assassyn.frontend import *
from opcodes import *
from scoreboard import *

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
        scoreboard:Array,
        RMT:Array
    ):
        self.timing = 'systolic'

        with Condition(self.rdata.valid()):
            data = self.rdata.pop()
            rd = self.rd.pop()
            log("mem.rdata        | 0x{:x}", data)
            with Condition(rd != Bits(5)(0)):
                log("mem.bypass       | x{:02} = 0x{:x}", rd, data)
                for index in range(SCOREBOARD.size):
                    Idx_mem=UInt(5)(index)
                    with Condition(scoreboard[index].sb_valid & ~scoreboard[index].sb_status):
                        log(" rs1_ready {:03x}  rs2_ready {:03x}  rs1_dep {:06} rs2_dep {:06} rmt[rd] {:06} ",\
                            scoreboard[index].rs1_ready,scoreboard[index].rs2_ready ,  scoreboard[index].rs1_dep ,\
                                scoreboard[index].rs2_dep ,RMT[rd]  )
                        rs1_get_data = ~scoreboard[index].rs1_ready & (scoreboard[index].rs1_dep== RMT[rd]) 
                        rs2_get_data = ~scoreboard[index].rs2_ready & (scoreboard[index].rs2_dep== RMT[rd] )
                        with Condition (rs1_get_data | rs2_get_data ):
                            scoreboard[index] =modify_entry_rs(scoreboard,index,rs1_get_data,rs2_get_data,data)
    
                RMT[rd]=Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)

        arg = self.rdata.valid().select(self.rdata.peek(), Bits(32)(0))
        writeback.async_called(mdata = arg)
