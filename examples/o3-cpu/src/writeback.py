from assassyn.frontend import *
from opcodes import * 
from scoreboard import *
class WriteBack(Module):
    
    def __init__(self):
        super().__init__(
            ports={ }, no_arbiter=True)

        self.name = 'W'

    @module.combinational
    def build(self, reg_file: Array ,scoreboard:Array, sb_head:Array,  signals_array:Array):
         
        s_head = sb_head[0]  
        wait_until(scoreboard['sb_status'][s_head]==Bits(2)(3))

        signals = signals_array[s_head]
        is_memory_read, result, rd, mdata,   mem_ext = \
            (signals.memory[0:0]), scoreboard['result'][s_head], signals.rd, \
            scoreboard['mdata'][s_head],  signals.mem_ext

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
            rmt_clear_index = s_head 
         
        scoreboard['sb_valid'][s_head] = Bits(1)(0)
        scoreboard['sb_status'][s_head] = Bits(2)(0)
         
        bypass_head = (
                (s_head.bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1) 
            ).bitcast(Bits(SCOREBOARD.Bit_size)) 
        )
        bypass_head = (bypass_head==NoDep).select(Bits(SCOREBOARD.Bit_size)(0),bypass_head)
        
        sb_head[0] = bypass_head 

        return rmt_clear_rd,rmt_clear_index
