from opcodes import *
from instructions import *

class SCOREBOARD:
    size = 8
    Bit_size = 4
    sizeI = UInt(32)(size)

NoDep=Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)

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
    result_ready=Bits(1),
    result = Bits(32),
    # 0: Issued; 1: Executed;  
    sb_status=Bits(2),
    signals=deocder_signals,
    fetch_addr=Bits(32),
    is_memory_read=Bits(1),
    mdata= Bits(32),
    is_csr= Bits(1),
    csr_id= Bits(4),
    csr_new= Bits(32),
    mem_ext= Bits(2)
    
)



def modify_entry_mdata_status(scoreboard,index,mdata, status ):
    mdata = mdata
    status= status  
    log("modify s[{:06}]  mdata {:08x}   status {}",index,mdata, status)
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
            result_ready=Bits(1)(1),
            result=scoreboard[index].result,
            sb_status= status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr,
            is_memory_read=scoreboard[index].is_memory_read,
            mdata=  mdata,
            is_csr= scoreboard[index].is_csr,
            csr_id= scoreboard[index].csr_id,
            csr_new= scoreboard[index].csr_new,
            mem_ext= scoreboard[index].mem_ext
)

def modify_entry_exe(scoreboard,index,result,status):
    status= status
    result=result
    log("modify index {}  result {}   status {}",index,result,status)
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
            result_ready=Bits(1)(1),
            result=result,
            sb_status= status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr,
            is_memory_read=scoreboard[index].is_memory_read,
            mdata=  scoreboard[index].mdata,
            is_csr= scoreboard[index].is_csr,
            csr_id= scoreboard[index].csr_id,
            csr_new= scoreboard[index].csr_new,
            mem_ext= scoreboard[index].mem_ext
)

def modify_entry_mem_status(scoreboard,index,is_memory_read ,result  ,is_csr  ,csr_id ,csr_new,mem_ext,status):
    is_memory_read=is_memory_read
    result=result
    is_csr=is_csr
    csr_id=csr_id
    csr_new=csr_new
    mem_ext=mem_ext
    status=status
    return scoreboard_entry.bundle(
           sb_valid=scoreboard[index].sb_valid,   
           rd=scoreboard[index].rd,
           rs1=scoreboard[index].rs1,
           rs2=scoreboard[index].rs2,
           rs1_ready=scoreboard[index].rs1_ready,
           rs2_ready=scoreboard[index].rs2_ready,
           rs1_value=scoreboard[index].rs1_value,
           rs2_value= scoreboard[index].rs2_value,
           rs1_dep=scoreboard[index].rs1_dep,
           rs2_dep=scoreboard[index].rs2_dep,
           result_ready=scoreboard[index].result_ready,
           result=result,
           sb_status= status,
           signals=scoreboard[index].signals,
           fetch_addr=scoreboard[index].fetch_addr,
           is_memory_read=is_memory_read,
           mdata=  scoreboard[index].mdata,
           is_csr= is_csr,
           csr_id= csr_id,
           csr_new=  csr_new,
           mem_ext= mem_ext
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
            result_ready=scoreboard[index].result_ready,
            result=scoreboard[index].result,
            sb_status= scoreboard[index].sb_status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr,
            is_memory_read=scoreboard[index].is_memory_read,
            mdata= scoreboard[index].mdata,
            is_csr= scoreboard[index].is_csr,
            csr_id= scoreboard[index].csr_id,
            csr_new= scoreboard[index].csr_new,
            mem_ext= scoreboard[index].mem_ext
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
            result_ready=scoreboard[index].result_ready,
            result=scoreboard[index].result,
            sb_status= sb_status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr,
            is_memory_read=scoreboard[index].is_memory_read,
            mdata= scoreboard[index].mdata,
            is_csr= scoreboard[index].is_csr,
            csr_id= scoreboard[index].csr_id,
            csr_new= scoreboard[index].csr_new,
            mem_ext= scoreboard[index].mem_ext
    )

def modify_entry_rs2(scoreboard,i,result):
    index = i
    result = result

    log("modify rs2 index {}  result {}",Bits(5)(index),result)
    return scoreboard_entry.bundle(
            sb_valid=scoreboard[index].sb_valid,   
            rd=scoreboard[index].rd,
            rs1=scoreboard[index].rs1,
            rs2=scoreboard[index].rs2,
            rs1_ready=scoreboard[index].rs1_ready,
            rs2_ready=Bits(1)(1),
            rs1_value=scoreboard[index].rs1_value,
            rs2_value=result,
            rs1_dep=scoreboard[index].rs1_dep,
            rs2_dep=scoreboard[index].rs2_dep,
            result_ready=scoreboard[index].result_ready,
            result=scoreboard[index].result,
            sb_status=scoreboard[index].sb_status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr,
            is_memory_read=scoreboard[index].is_memory_read,
            mdata= scoreboard[index].mdata,
            is_csr= scoreboard[index].is_csr,
            csr_id= scoreboard[index].csr_id,
            csr_new= scoreboard[index].csr_new,
            mem_ext= scoreboard[index].mem_ext
)


def modify_entry_rs1(scoreboard,i,result):
    index = i
    result = result
    log("modify rs1 index {}  result {}",Bits(5)(index),result)
    return scoreboard_entry.bundle(
            sb_valid=scoreboard[index].sb_valid,   
            rd=scoreboard[index].rd,
            rs1=scoreboard[index].rs1,
            rs2=scoreboard[index].rs2,
            rs1_ready=Bits(1)(1),
            rs2_ready=scoreboard[index].rs2_ready,
            rs1_value=result,
            rs2_value=scoreboard[index].rs2_value,
            rs1_dep=scoreboard[index].rs1_dep,
            rs2_dep=scoreboard[index].rs2_dep,
            result_ready=scoreboard[index].result_ready,
            result=scoreboard[index].result,
            sb_status=scoreboard[index].sb_status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr,
            is_memory_read=scoreboard[index].is_memory_read,
            mdata= scoreboard[index].mdata,
            is_csr= scoreboard[index].is_csr,
            csr_id= scoreboard[index].csr_id,
            csr_new= scoreboard[index].csr_new,
            mem_ext= scoreboard[index].mem_ext
)


def modify_entry_rs(scoreboard,index,rs1_ready_update,rs2_ready_update):
    rs1_s = rs1_ready_update.select(Bits(1)(1),scoreboard[index].rs1_ready)
    rs2_s = rs2_ready_update.select(Bits(1)(1),scoreboard[index].rs2_ready)
    rs1_dep=scoreboard[index].rs1_dep
    rs1_update_value = (scoreboard[rs1_dep].is_memory_read).select(scoreboard[rs1_dep].mdata , scoreboard[rs1_dep].result)
    rs1_value = rs1_ready_update.select(rs1_update_value,scoreboard[index].rs1_value)
    rs2_dep=scoreboard[index].rs2_dep
    rs2_update_value = (scoreboard[rs2_dep].is_memory_read).select(scoreboard[rs2_dep].mdata , scoreboard[rs2_dep].result)
    rs2_value = rs2_ready_update.select(rs2_update_value,scoreboard[index].rs2_value)
    log("scoreboard[rs1_dep].result {}",scoreboard[rs1_dep].result)
    log("rs1  dep {}  is_mem_r {} rs1_update_value {}  ", rs1_dep,scoreboard[rs1_dep].is_memory_read,rs1_update_value)
    log("modify index {} rs1_ready {} ,rs1_value {}   rs2_ready {}, rs2_value {}",Bits(32)(index),rs1_s,rs1_value,rs2_s,rs2_value)
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
            result_ready=scoreboard[index].result_ready,
            result=scoreboard[index].result,
            sb_status=scoreboard[index].sb_status ,
            signals=scoreboard[index].signals,
            fetch_addr=scoreboard[index].fetch_addr,
            is_memory_read=scoreboard[index].is_memory_read,
            mdata= scoreboard[index].mdata,
            is_csr= scoreboard[index].is_csr,
            csr_id= scoreboard[index].csr_id,
            csr_new= scoreboard[index].csr_new,
            mem_ext= scoreboard[index].mem_ext
) 


def modify_entry_sb_status(new_entry):
     return scoreboard_entry.bundle(
            sb_valid=new_entry.sb_valid,   
            rd=new_entry.rd,
            rs1=new_entry.rs1,
            rs2=new_entry.rs2,
            rs1_ready=new_entry.rs1_ready,
            rs2_ready=new_entry.rs2_ready,
            rs1_value=new_entry.rs1_value,
            rs2_value=new_entry.rs2_value,
            rs1_dep=new_entry.rs1_dep,
            rs2_dep=new_entry.rs2_dep,
            result_ready=new_entry.result_ready,
            result=new_entry.result,
            sb_status=Bits(2)(1) ,
            signals=new_entry.signals,
            fetch_addr=new_entry.fetch_addr,
            is_memory_read=new_entry.is_memory_read,
            mdata= new_entry.mdata,
            is_csr= new_entry.is_csr,
            csr_id= new_entry.csr_id,
            csr_new= new_entry.csr_new,
            mem_ext= new_entry.mem_ext
) 


def add_entry(signals,scoreboard,Index,RMT,reg_file,fetch_addr,mem_index,ex_index,e_data,m_data):
  
        sb_valid=Bits(1)(1)
        rd=signals.rd
        rs1=signals.rs1
        rs2=signals.rs2
        
        entry_rs1 = scoreboard[RMT[rs1]]
        entry_rs2 = scoreboard[RMT[rs2]]
        rs1_result_valid =  ((entry_rs1.sb_status== Bits(2)(3) )& (RMT[rs1] != NoDep))
        rs1_value_prefetch = (entry_rs1.is_memory_read).select( entry_rs1.mdata , entry_rs1.result)
        rs1_ready=(signals.rs1_valid & (RMT[rs1] != NoDep)&(~rs1_result_valid)).select(Bits(1)(0), Bits(1)(1))
        rs1_dep=(~rs1_ready).select(RMT[rs1],NoDep)
        rs1_value=(rs1_result_valid).select(rs1_value_prefetch,reg_file[signals.rs1])
        log("rs1_valid {} rs1_value_prefetch {} rs1_ready {} rs1_dep {} rs1_value {}",signals.rs1_valid,rs1_value_prefetch,rs1_ready,rs1_dep,rs1_value)
        
        rs2_result_valid =  ((entry_rs2.sb_status== Bits(2)(3) )& (RMT[rs2] != NoDep))
        rs2_value_prefetch = (entry_rs2.is_memory_read).select( entry_rs2.mdata , entry_rs2.result)
        rs2_ready=(signals.rs2_valid & (RMT[rs2] != NoDep)&(~rs2_result_valid)).select(Bits(1)(0), Bits(1)(1))
        rs2_dep=(~rs2_ready).select(RMT[rs2],NoDep)
        rs2_value=(rs2_result_valid).select(rs2_value_prefetch,reg_file[signals.rs2])
        log("rs2_valid {} rs2_value_prefetch {} rs2_ready {} rs2_dep {} rs2_value {}",signals.rs2_valid,rs2_value_prefetch,rs2_ready,rs2_dep,rs2_value)
        
        m_is_memory_read = deocder_signals.view(scoreboard[mem_index].signals).memory[0:0]
        e_is_memory_read = deocder_signals.view(scoreboard[ex_index].signals).memory[0:0]

        mem_valid = (mem_index != NoDep)&(m_is_memory_read)
        rs1_value = (( RMT[rs1]==mem_index)&(mem_valid)&(~rs1_ready)).select( m_data,rs1_value )
        rs2_value = (( RMT[rs2]==mem_index)&(mem_valid)&(~rs2_ready)).select( m_data,rs2_value )
        rs1_ready = (((RMT[rs1]==mem_index)&mem_valid&(~rs1_ready))).select(Bits(1)(1),rs1_ready)
        rs2_ready = (((RMT[rs2]==mem_index)&mem_valid&(~rs2_ready))).select(Bits(1)(1),rs2_ready)
        log("rs1_value {} rs2_value {} rs1_ready {} rs2_ready {}",rs1_value,rs2_value,rs1_ready,rs2_ready)

        exe_valid = ( ex_index != NoDep)&(~e_is_memory_read)&(scoreboard[ex_index].rd!=Bits(5)(0))
        rs1_value = (( RMT[rs1]==ex_index)&exe_valid&(~rs1_ready)).select( e_data,rs1_value )
        rs2_value = (( RMT[rs2]==ex_index)&exe_valid&(~rs2_ready)).select( e_data,rs2_value )
        rs1_ready = (((RMT[rs1]==ex_index)&exe_valid)&(~rs1_ready)).select(Bits(1)(1),rs1_ready)
        rs2_ready = (((RMT[rs2]==ex_index)&exe_valid)&(~rs2_ready)).select(Bits(1)(1),rs2_ready)
        log("rs1_value {} rs2_value {} rs1_ready {} rs2_ready {}",rs1_value,rs2_value,rs1_ready,rs2_ready)
     
        sb_status=Bits(2)(0)
        

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
            result_ready=Bits(1)(0),
            result=Bits(32)(0),
            sb_status=sb_status ,
            signals=signals,
            fetch_addr=fetch_addr,
            is_memory_read=Bits(1)(0),
            mdata= Bits(32)(0),
            is_csr= Bits(1)(0),
            csr_id= Bits(4)(0),
            csr_new= Bits(32)(0),
            mem_ext= Bits(2)(0)
) 


