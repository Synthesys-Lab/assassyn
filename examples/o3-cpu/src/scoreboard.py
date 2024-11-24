from assassyn.frontend import *
from opcodes import *
from instructions import *

class SCOREBOARD:
    size = 16
    Bit_size = 5
    sizeI = UInt(32)(size)

#Todo: modify the fields 
#leave it like this for now
scoreboard_entry = Record(
    sb_valid=Bits(1),
    
    rd=Bits(5),
    rs1=Bits(5),
    rs2=Bits(5),
    
    rs1_ready=Bits(1),
    rs2_ready=Bits(1),
    rs1_value=Bits(32),
    rs2_value=Bits(32),
    # Dependent entry on rs1, MAX
    rs1_dep=Bits(SCOREBOARD.Bit_size),
    rs2_dep=Bits(SCOREBOARD.Bit_size),
    # 0: Issued; 1: Executed;  
    # set valid to 0 when write back completed other wise keep 1
    sb_status=Bits(1),
    signals=deocder_signals,
    fetch_addr=Bits(32)
)


def modify_entry_valid(scoreboard,index,valid):
    valid = valid
    return scoreboard_entry.bundle(
            sb_valid=valid,   
            rd=scoreboard[index].rd,
            rs1=scoreboard[index].rs1,
            rs2=scoreboard[index].rs2,
            rs1_ready=scoreboard[index].rs1_ready,
            rs2_ready=scoreboard[index].rs2_ready,
            rs1_value=scoreboard[index].rs1_value,
            rs2_value=scoreboard[index].rs2_value,
            rs1_dep=scoreboard[index].rs1_dep,
            rs2_dep=scoreboard[index].rs2_dep,
            sb_status= scoreboard[index].sb_status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr
)

def modify_entry_status(scoreboard,index,status):
    log("write s[{:06}]  status {:04}",index,status)
    sb_status = status
    return scoreboard_entry.bundle(
            sb_valid=scoreboard[index].sb_valid,   
            rd=scoreboard[index].rd,
            rs1=scoreboard[index].rs1,
            rs2=scoreboard[index].rs2,
            rs1_ready=scoreboard[index].rs1_ready,
            rs2_ready=scoreboard[index].rs2_ready,
            rs1_value=scoreboard[index].rs1_value,
            rs2_value=scoreboard[index].rs2_value,
            rs1_dep=scoreboard[index].rs1_dep,
            rs2_dep=scoreboard[index].rs2_dep,
            sb_status= sb_status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr
    )
    
def modify_entry_rs(scoreboard,index,rs1_ready_update,rs2_ready_update, rs_value):
    rs1_s = rs1_ready_update.select(Bits(1)(1),scoreboard[index].rs1_ready)
    rs2_s = rs2_ready_update.select(Bits(1)(1),scoreboard[index].rs2_ready)
    rs1_value = rs1_ready_update.select(rs_value, scoreboard[index].rs1_value)
    rs2_value = rs2_ready_update.select(rs_value, scoreboard[index].rs2_value)
    
    return scoreboard_entry.bundle(
            sb_valid=scoreboard[index].sb_valid,   
            rd=scoreboard[index].rd,
            rs1=scoreboard[index].rs1,
            rs2=scoreboard[index].rs2,
            rs1_ready=rs1_s,
            rs2_ready=rs2_s,
            rs1_value=rs1_value,
            rs2_value=rs2_value,
            rs1_dep=scoreboard[index].rs1_dep,
            rs2_dep=scoreboard[index].rs2_dep,
            sb_status=scoreboard[index].sb_status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr
)


def add_entry(signals,scoreboard,Index,RMT,reg_file,fetch_addr):

        sb_valid=Bits(1)(1)
        rd=signals.rd
        rs1=signals.rs1
        rs2=signals.rs2
        
        NoDep = Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
        rs1_ready=(signals.rs1_valid & (RMT[rs1] != NoDep)).select(Bits(1)(0), Bits(1)(1))
        rs1_dep=(~rs1_ready).select(RMT[rs1],NoDep)
        rs1_value=(rs1_ready).select(reg_file[signals.rs1],Bits(32)(0))
        
        rs2_ready=(signals.rs2_valid & (RMT[rs2] != NoDep)).select(Bits(1)(0), Bits(1)(1))
        rs2_dep=(~rs2_ready).select(RMT[rs2],NoDep)
        rs2_value=(rs2_ready).select(reg_file[signals.rs2],Bits(32)(0))
        sb_status=Bits(1)(0)
        log("update current sb_status 0 for index {}",Index)
       
        log("Bundle values: valid={},  rd={}, rs1={}, rs2={}, rs1_ready={}, rs2_ready={}, rs1_value={}, rs2_value={},\
             rs1_dep={}, rs2_dep={}, sb_status={} ",\
        sb_valid, rd, rs1, rs2, rs1_ready, rs2_ready,\
        rs1_value,rs2_value,rs1_dep,rs2_dep,sb_status )

        return scoreboard_entry.bundle(
            sb_valid=sb_valid,   
            rd=rd,
            rs1=rs1,
            rs2=rs2,
            rs1_ready=rs1_ready,
            rs2_ready=rs2_ready,
            rs1_value=rs1_value,
            rs2_value=rs2_value,
            rs1_dep=rs1_dep,
            rs2_dep=rs2_dep,
            sb_status=sb_status ,
            signals=signals,
            fetch_addr=fetch_addr
)

