from opcodes import *
from instructions import *

class SCOREBOARD:
    size = 6
    init_size = size+1
    Bit_size = 3
    sizeI = UInt(8)(size)

NoDep=Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)


scoreboard_entry = Record( 
    rs1_ready=Bits(1),
    rs2_ready=Bits(1),
    rs1_value=Bits(32),
    rs2_value=Bits(32), 
    rs1_dep=Bits(SCOREBOARD.Bit_size),
    rs2_dep=Bits(SCOREBOARD.Bit_size),    
    fetch_addr=Bits(32), 
)

# mem_ext = signals.mem_ext,
# is_csr = signals.csr_write,
  
 
def add_entry(signals, scoreboard,signals_array, RMT, reg_file, fetch_addr, mem_index, ex_index, e_data, m_data):
     
    rs1 = signals.rs1
    rs2 = signals.rs2
    
    entry_rs1 = RMT[rs1] 
    entry_rs2 = RMT[rs2]
    rs1_dp = RMT[rs1] != NoDep
    rs2_dp = RMT[rs2] != NoDep
    rs1_rd = signals_array[entry_rs1].rd == rs1
    rs2_rd = signals_array[entry_rs2].rd == rs2
    rs1_dp_valid = scoreboard['sb_valid'][entry_rs1]
    rs2_dp_valid =  scoreboard['sb_valid'][entry_rs2]

    rs1_mem_vd = RMT[rs1] == mem_index
    rs2_mem_vd = RMT[rs2] == mem_index

    rs1_ex_vd = RMT[rs1] == ex_index
    rs2_ex_vd = RMT[rs2] == ex_index

    rs1_result_valid = ((scoreboard['sb_status'][entry_rs1] == Bits(2)(3)) & 
                         (rs1_dp) & 
                         rs1_dp_valid & 
                         (rs1_rd))
     
    rs1_value_prefetch = (signals_array[entry_rs1].memory[0:0]).select(scoreboard['mdata'][entry_rs1], scoreboard['result'][entry_rs1])
    rs1_ready = (signals.rs1_valid & (rs1_dp) & (~rs1_result_valid) & 
                 rs1_dp_valid & 
                 (rs1_rd)).select(Bits(1)(0), Bits(1)(1))
     
    rs1_dep = (~rs1_ready).select(RMT[rs1], NoDep)
    rs1_value = rs1_result_valid.select(rs1_value_prefetch, reg_file[signals.rs1])

    rs2_result_valid = ((scoreboard['sb_status'][entry_rs2] == Bits(2)(3)) & 
                         (rs2_dp) & 
                         rs2_dp_valid & 
                         (rs2_rd))
     
    rs2_value_prefetch = (signals_array[entry_rs2].memory[0:0]).select(scoreboard['mdata'][entry_rs2], scoreboard['result'][entry_rs2])
    rs2_ready = (signals.rs2_valid & (rs2_dp) & (~rs2_result_valid) & 
                 rs2_dp_valid & 
                 (rs2_rd)).select(Bits(1)(0), Bits(1)(1))

    rs2_dep = (~rs2_ready).select(RMT[rs2], NoDep)
    rs2_value = rs2_result_valid.select(rs2_value_prefetch, reg_file[signals.rs2])

    m_is_memory_read = signals_array[mem_index].memory[0:0]
    e_is_memory_read = signals_array[ex_index].memory[0:0]

    mem_valid = (mem_index != NoDep) & m_is_memory_read
    exe_valid = (ex_index != NoDep) & (~e_is_memory_read) & (signals_array[ex_index].rd != Bits(5)(0))
 
    mem_rs1_update = (rs1_mem_vd) & mem_valid & (~rs1_ready) 
    mem_rs2_update = (rs2_mem_vd) & mem_valid & (~rs2_ready)
    rs1_value = (mem_rs1_update &  rs1_dp_valid).select(m_data, rs1_value)

    rs2_value = (mem_rs2_update &   rs2_dp_valid).select(m_data, rs2_value)
     
    ex_rs1_update = ((rs1_ex_vd & exe_valid) & (~rs1_ready))
    ex_rs2_update = (rs2_ex_vd & exe_valid & (~rs2_ready))
    
    
    
    rs1_value = (ex_rs1_update).select(e_data, rs1_value)
    rs2_value = ex_rs2_update.select(e_data, rs2_value)
    
    rs1_ready = (mem_rs1_update  |  ex_rs1_update).select(Bits(1)(1), rs1_ready)
    
    rs2_ready = (mem_rs2_update |  ex_rs2_update ).select(Bits(1)(1), rs2_ready)
  
    
    return scoreboard_entry.bundle( 
        rs1_ready=rs1_ready,
        rs2_ready=rs2_ready,
        rs1_value=rs1_value,
        rs2_value=rs2_value,
        rs1_dep=rs1_dep,
        rs2_dep=rs2_dep,    
        fetch_addr=fetch_addr 
    )


