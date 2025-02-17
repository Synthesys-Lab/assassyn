from assassyn.frontend import *
from opcodes import * 
from scoreboard import *
class WriteBack(Module):
    
    def __init__(self):
        super().__init__(
            ports={}, no_arbiter=True)

        self.name = 'W'

    @module.combinational
    def build(self, reg_file: Array , csr_file: Array,scoreboard:Array,RMT:Array,sb_head:Array):
        log("in wb")
        
        wb_valid =(scoreboard[sb_head[0]].sb_status==Bits(2)(3)) 
        wait_until(wb_valid)
        entry = scoreboard[sb_head[0]]
        is_memory_read, result, rd, mdata , is_csr , csr_id , csr_new , mem_ext= \
            entry.is_memory_read,entry.result,entry.rd,entry.mdata,entry.is_csr,entry.csr_id,entry.csr_new,entry.mem_ext 
        data_cut = Bits(32)(0)
        sign = mdata[7:7]
        ext = sign.select(Bits(24)(0xffffff), Bits(24)(0))
        data_cut = mem_ext[1:1].select( Bits(24)(0).concat(mdata[0:7]) , ext.concat(mdata[0:7]) )
        data = is_memory_read.select(mdata, result)
        data = mem_ext[0:0].select( data_cut ,data)
        with Condition((rd != Bits(5)(0))):
            log("writeback        | x{:02}          | 0x{:08x}", rd, data)
            reg_file[rd] = data
            rmt_clear_rd = rd
            rmt_clear_index = sb_head[0]
        with Condition(is_csr):
            log("writeback        | csr[{:02}]       | 0x{:08x}", csr_id, csr_new)
            csr_file[csr_id] = csr_new
         
        scoreboard[sb_head[0]] = modify_entry_valid(scoreboard,sb_head[0],Bits(1)(0))
        sb_head[0] = (
            (sb_head[0].bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1))
        ).bitcast(Bits(SCOREBOARD.Bit_size)) & (Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size - 1))
        log("Update sb_head {:07} ",sb_head[0])
        
        return rd,rmt_clear_rd,rmt_clear_index
