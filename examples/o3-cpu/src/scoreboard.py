from opcodes import *
from instructions import *

class SCOREBOARD:
    size = 4
    init_size = 5
    Bit_size = 3
    sizeI = UInt(32)(size)

NoDep=Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)


scoreboard_entry = Record(
    sb_valid=Bits(1), 
    rs1_ready=Bits(1),
    rs2_ready=Bits(1),
    rs1_value=Bits(32),
    rs2_value=Bits(32), 
    rs1_dep=Bits(SCOREBOARD.Bit_size),
    rs2_dep=Bits(SCOREBOARD.Bit_size),  
    sb_status=Bits(2),
    signals=decoder_signals,
    fetch_addr=Bits(32), 
)

# mem_ext = signals.mem_ext,
# is_csr = signals.csr_write,
  
 
def add_entry(signals, scoreboard, RMT, reg_file, fetch_addr, mem_index, ex_index, e_data, m_data):
    sb_valid = Bits(1)(1) 
    rs1 = signals.rs1
    rs2 = signals.rs2
 
    entry_rs1 = RMT[rs1]
    entry_rs2 = RMT[rs2]
 
    rs1_result_valid = ((scoreboard['sb_status'][entry_rs1] == Bits(2)(3)) & 
                         (RMT[rs1] != NoDep) & 
                         scoreboard['sb_valid'][entry_rs1] & 
                         (scoreboard['signals'][entry_rs1].rd == rs1))
    
    rs1_value_prefetch = scoreboard['is_memory_read'][entry_rs1].select(scoreboard['mdata'][entry_rs1], scoreboard['result'][entry_rs1])
    rs1_ready = (signals.rs1_valid & (RMT[rs1] != NoDep) & (~rs1_result_valid) & 
                 scoreboard['sb_valid'][entry_rs1] & 
                 (scoreboard['signals'][entry_rs1].rd == rs1)).select(Bits(1)(0), Bits(1)(1))
 
    rs1_dep = (~rs1_ready).select(RMT[rs1], NoDep)
    rs1_value = rs1_result_valid.select(rs1_value_prefetch, reg_file[signals.rs1])

    rs2_result_valid = ((scoreboard['sb_status'][entry_rs2] == Bits(2)(3)) & 
                         (RMT[rs2] != NoDep) & 
                         scoreboard['sb_valid'][entry_rs2] & 
                         (scoreboard['signals'][entry_rs2].rd == rs2))
    
    rs2_value_prefetch = scoreboard['is_memory_read'][entry_rs2].select(scoreboard['mdata'][entry_rs2], scoreboard['result'][entry_rs2])
    rs2_ready = (signals.rs2_valid & (RMT[rs2] != NoDep) & (~rs2_result_valid) & 
                 scoreboard['sb_valid'][entry_rs2] & 
                 (scoreboard['signals'][entry_rs2].rd == rs2)).select(Bits(1)(0), Bits(1)(1))

    rs2_dep = (~rs2_ready).select(RMT[rs2], NoDep)
    rs2_value = rs2_result_valid.select(rs2_value_prefetch, reg_file[signals.rs2])

    m_is_memory_read = scoreboard['signals'][mem_index].memory[0:0]
    e_is_memory_read = scoreboard['signals'][ex_index].memory[0:0]

    mem_valid = (mem_index != NoDep) & m_is_memory_read
    rs1_value = ((RMT[rs1] == mem_index) & mem_valid & (~rs1_ready) & 
                 scoreboard['sb_valid'][entry_rs1]).select(m_data, rs1_value)
    
    rs2_value = ((RMT[rs2] == mem_index) & mem_valid & (~rs2_ready) & 
                 scoreboard['sb_valid'][entry_rs2]).select(m_data, rs2_value)
    
    
    log("rs1_value {} rs2_value {} rs1_ready {} rs2_ready {}", rs1_value, rs2_value, rs1_ready, rs2_ready)

    exe_valid = (ex_index != NoDep) & (~e_is_memory_read) & (scoreboard['signals'][ex_index].rd != Bits(5)(0))
    rs1_value = ((RMT[rs1] == ex_index) & exe_valid & (~rs1_ready)).select(e_data, rs1_value)
    rs2_value = ((RMT[rs2] == ex_index) & exe_valid & (~rs2_ready)).select(e_data, rs2_value)
    
    rs1_ready = (((RMT[rs1] == mem_index) & mem_valid & (~rs1_ready)  ) | (((RMT[rs1] == ex_index) & exe_valid) & (~rs1_ready)) ).select(Bits(1)(1), rs1_ready)
    
    rs2_ready = ((RMT[rs2] == mem_index) & mem_valid & (~rs2_ready) |  (((RMT[rs2] == ex_index) & exe_valid) & (~rs2_ready)) ).select(Bits(1)(1), rs2_ready)
 
    log("rs1_value {} rs2_value {} rs1_ready {} rs2_ready {}", rs1_value, rs2_value, rs1_ready, rs2_ready)

    sb_status = Bits(2)(0)

    return scoreboard_entry.bundle(
        sb_valid=sb_valid,    
        rs1_ready=rs1_ready,
        rs2_ready=rs2_ready,
        rs1_value=rs1_value,
        rs2_value=rs2_value,
        rs1_dep=rs1_dep,
        rs2_dep=rs2_dep,  
        sb_status=sb_status,
        signals=signals,
        fetch_addr=fetch_addr 
    )


