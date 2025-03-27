from opcodes import *
from instructions import *

class SCOREBOARD:
    size = 5
    init_size = size+1
    Bit_size = 3
    sizeI = UInt(8)(size)

NoDep=Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)

 
 
 
def call_rs(rs1,rs2,rs1_valid,rs2_valid, scoreboard,signals_array, entry_rs1,entry_rs2, mem_index, ex_index):
       
    rs1_ready = ( rs1_valid   & (~ (scoreboard['sb_status'][entry_rs1] == Bits(2)(3))  ) & 
                 (scoreboard['sb_valid'][entry_rs1]) & 
                 (signals_array[entry_rs1].rd == rs1)).select(Bits(1)(0), Bits(1)(1))
      
      
    rs2_ready = ( rs2_valid  & (~ (scoreboard['sb_status'][entry_rs2] == Bits(2)(3)) ) & 
                 ( scoreboard['sb_valid'][entry_rs2]) & 
                 (signals_array[entry_rs2].rd == rs2)).select(Bits(1)(0), Bits(1)(1))
  
    
    rs1_ready = ((entry_rs1 == mem_index)  |  (entry_rs1 == ex_index) ).select(Bits(1)(1), rs1_ready) 
    rs2_ready = ((entry_rs2 == mem_index)  |  (entry_rs2 == ex_index)   ).select(Bits(1)(1), rs2_ready)
    
    return rs1_ready, rs2_ready
    


