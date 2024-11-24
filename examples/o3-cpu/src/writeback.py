from assassyn.frontend import *
from opcodes import * 
from scoreboard import *
class WriteBack(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'is_memory_read': Port(Bits(1)),
                'result': Port(Bits(32)),
                'rd': Port(Bits(5)),
                'mdata': Port(Bits(32)),
                'is_csr': Port(Bits(1)),
                'csr_id': Port(Bits(4)),
                'csr_new': Port(Bits(32)),
                'mem_ext': Port(Bits(2)),
                'index':Port(Bits(SCOREBOARD.Bit_size))
            }, no_arbiter=True)

        self.name = 'W'

    @module.combinational
    def build(self, reg_file: Array , csr_file: Array,scoreboard:Array,RMT:Array,sb_head:Array):

        is_memory_read, result, rd, mdata , is_csr , csr_id , csr_new , mem_ext,index= self.pop_all_ports(True)
        wb_valid =( sb_head[0]==index)
        log("sb_head {:07}  index {:07} ",sb_head[0], index)
        wait_until(wb_valid)
        with Condition(wb_valid):
            log("Wb_valid")
            data_cut = Bits(32)(0)
            sign = mdata[7:7]
            ext = sign.select(Bits(24)(0xffffff), Bits(24)(0))
            data_cut = mem_ext[1:1].select( Bits(24)(0).concat(mdata[0:7]) , ext.concat(mdata[0:7]) )

            data = is_memory_read.select(mdata, result)
            data = mem_ext[0:0].select( data_cut ,data)

            with Condition((rd != Bits(5)(0))):
                log("writeback        | x{:02}          | 0x{:08x}", rd, data)
                reg_file[rd] = data

                
                


            with Condition(is_csr):
                log("writeback        | csr[{:02}]       | 0x{:08x}", csr_id, csr_new)
                csr_file[csr_id] = csr_new

             
        

            scoreboard[index] = modify_entry_valid(scoreboard,index,Bits(1)(0))
            sb_head[0]= (((sb_head[0].bitcast(UInt(32)) )+UInt(32)(1) )%SCOREBOARD.sizeI ).bitcast(Bits(SCOREBOARD.Bit_size))
            
            log("Update sb_head {:07} ",sb_head[0])
            log("out writeback")
            #Todo:make it commit in order  
            return rd
