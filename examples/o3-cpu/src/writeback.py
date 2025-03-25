from assassyn.frontend import *
from opcodes import * 
from scoreboard import *
class WriteBack(Module):
    
    def __init__(self):
        super().__init__(
            ports={}, no_arbiter=True)

        self.name = 'W'

    @module.combinational
    def build(self, reg_file: Array ,sb_valid_array:Array,sb_status_array:Value, sb_head:Array,  \
          result_array:Array , \
                    mdata_array:Array ,  
                    signals_array:Array,):
          
        wb_valid =(sb_status_array[sb_head[0]]==Bits(2)(3)) 
        wait_until(wb_valid)
         
        is_memory_read, result, rd, mdata,   mem_ext = \
            (signals_array[sb_head[0]].memory[0:0]), result_array[sb_head[0]], signals_array[sb_head[0]].rd, \
            mdata_array[sb_head[0]],  \
            signals_array[sb_head[0]].mem_ext

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
            
        sb_valid_array[sb_head[0]] = Bits(1)(0)
        sb_status_array[sb_head[0]] = Bits(2)(0)
         
        bypass_tail = (
                (sb_head[0].bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1) 
            ).bitcast(Bits(SCOREBOARD.Bit_size)) 
        )
        bypass_tail = (bypass_tail==NoDep).select(Bits(SCOREBOARD.Bit_size)(0),bypass_tail)
        
        sb_head[0] = bypass_tail
        return rmt_clear_rd,rmt_clear_index
