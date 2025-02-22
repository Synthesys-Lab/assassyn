'''  
'''

import os
import shutil


from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

from opcodes import *
from decoder import *
from writeback import *
from memory_access import *
from scoreboard import *


offset = UInt(32)(0)
current_path = os.path.dirname(os.path.abspath(__file__))
workspace = f'{current_path}/.workspace/'


class Execution(Module):
    
    def __init__(self):
        super().__init__(
            ports={ 
                'sb_index':Port(Bits(SCOREBOARD.Bit_size)) 
            })
        self.name = "E"

    @module.combinational
    def build(
        self,  
        rf: Array, 
        csr_f: Array,
        memory: Module, 
        writeback: Module,
        data: str,
        depth_log: int,
        scoreboard:Array, 
        offset_reg: Array
        ):

        csr_id = Bits(4)(0)
         
        sb_index = self.sb_index.pop()
        # rs1_value,rs2_value,signals, fetch_addr = self.pop_all_ports(False)
        rs1_value=scoreboard['rs1_value'][sb_index]
        rs2_value=scoreboard['rs2_value'][sb_index]
        signals=scoreboard['signals'][sb_index]
        fetch_addr=scoreboard['fetch_addr'][sb_index]
        rs1 = signals.rs1 
        rd = signals.rd
         
        raw_id = [(3860, 9), (773, 1) ,(1860, 15) , (384,10) , (944 , 11) , (928 , 12) , (772 , 4 ) , (770 ,13),(771,14),(768,8) ,(833,2)]
        #mtvec 1 mepc 2 mcause 3 mie 4 mip 5 mtval 6 mscratc 7 msb_status 8 mhartid 9 satp 10 pmpaddr0 11  pmpcfg0 12 medeleg 13 mideleg 14 unkonwn 15

        csr_id = Bits(4)(0)
        for i, j in raw_id:
            csr_id = (signals.imm[0:11] == Bits(12)(i)).select(Bits(4)(j), csr_id)
            csr_id = signals.is_mepc.select(Bits(4)(2), csr_id)

        is_csr = Bits(1)(0)
        is_csr = signals.csr_read | signals.csr_write
        csr_new = Bits(32)(0)
        csr_new = signals.csr_write.select( rf[rs1] , csr_new)
        csr_new = signals.is_zimm.select(concat(Bits(27)(0),rs1), csr_new)

        with Condition(is_csr):
            log("csr_id: {} | new: {:08x} |", csr_id, csr_new)

        # TODO(@were): This is a hack to avoid post wait_until checks.
        rd = signals.rd

        is_ebreak = signals.rs1_valid & signals.imm_valid & \
                    ((signals.imm == Bits(32)(1)) | (signals.imm == Bits(32)(0))) & \
                    (signals.alu == Bits(16)(0))
        with Condition(is_ebreak):
            log('ebreak | halt | ecall')
            finish()


        is_trap = signals.is_branch & \
                  signals.is_offset_br & \
                  signals.imm_valid & \
                  (signals.imm == Bits(32)(0)) & \
                  (signals.cond == Bits(RV32I_ALU.CNT)(1 << RV32I_ALU.ALU_TRUE)) & \
                  (signals.alu == Bits(RV32I_ALU.CNT)(1 << RV32I_ALU.ALU_ADD))
        with Condition(is_trap):
            log('trap')
            finish()


        a = rs1_value
        a = signals.csr_write.select(Bits(32)(0), a)


        b = rs2_value
        b = is_csr.select(csr_f[csr_id], b)
        


        # TODO: To support `auipc`, is_branch will be separated into `is_branch` and `is_pc_calc`.
        alu_a = (signals.is_offset_br | signals.is_pc_calc).select(fetch_addr, a)
        alu_b = signals.imm_valid.select(signals.imm, b)

        results = [Bits(32)(0)] * RV32I_ALU.CNT

        adder_result = (alu_a.bitcast(Int(32)) + alu_b.bitcast(Int(32))).bitcast(Bits(32))
        le_result = (a.bitcast(Int(32)) < b.bitcast(Int(32))).select(Bits(32)(1), Bits(32)(0))
        eq_result = (a == b).select(Bits(32)(1), Bits(32)(0))
        leu_result = (a < b).select(Bits(32)(1), Bits(32)(0))
        sra_signed_result = (a.bitcast(Int(32)) >> alu_b[0:4].bitcast(Int(5))).bitcast(Bits(32))
        sub_result = (a.bitcast(Int(32)) - b.bitcast(Int(32))).bitcast(Bits(32))

        results[RV32I_ALU.ALU_ADD] = adder_result
        results[RV32I_ALU.ALU_SUB] = sub_result
        results[RV32I_ALU.ALU_CMP_LT] = le_result
        results[RV32I_ALU.ALU_CMP_EQ] = eq_result
        results[RV32I_ALU.ALU_CMP_LTU] = leu_result
        results[RV32I_ALU.ALU_XOR] = a ^ alu_b
        results[RV32I_ALU.ALU_OR] = a | b
        results[RV32I_ALU.ALU_ORI] = a | alu_b
        results[RV32I_ALU.ALU_AND] = a & alu_b
        results[RV32I_ALU.ALU_TRUE] = Bits(32)(1)
        results[RV32I_ALU.ALU_SLL] = a << alu_b[0:4]
        results[RV32I_ALU.ALU_SRA] = sra_signed_result 
        results[RV32I_ALU.ALU_SRA_U] = a >> alu_b[0:4]

        # TODO: Fix this bullshit.
        alu = signals.alu
        result = alu.select1hot(*results)

        log('pc: 0x{:08x}   |is_offset_br: {}| is_pc_calc: {}|', fetch_addr, signals.is_offset_br, signals.is_pc_calc)
        log("0x{:08x}       | a: {:08x}  | b: {:08x}   | imm: {:08x} | result: {:08x}", alu, a, b, signals.imm, result)
        log("0x{:08x}       |a.a:{:08x}  |a.b:{:08x}   | res: {:08x} |", alu, alu_a, alu_b, result)

        condition = signals.cond.select1hot(*results)
        condition = signals.flip.select(~condition, condition)

        memory_read = signals.memory[0:0]
        memory_write = signals.memory[1:1]

        # TODO: Make this stricter later.
        produced_by_exec = ~memory_read & (rd != Bits(5)(0))
         
        pc0 = (fetch_addr.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
         
        with Condition(produced_by_exec): 
            res = signals.link_pc.select(pc0, result) 
            scoreboard['result'][sb_index] = res
            scoreboard['sb_status'][sb_index] = Bits(2)(3) 
            scoreboard['is_memory_read'][sb_index] = Bits(1)(0)
             
            ex_update = sb_index
            ex_data = res
            
             
        with Condition(signals.is_branch):
            br_dest = condition[0:0].select(result, pc0)
            execution_index = sb_index 
            log("condition: {}.a.b | a: {:08x}  | b: {:08x}   |", condition[0:0], result, pc0)
            predict_wrong = condition[0:0].select(Bits(1)(0),Bits(1)(1)) 
            predict_wrong = (signals.is_branch & (~signals.is_offset_br)&signals.link_pc).select(Bits(1)(1),predict_wrong) 
           
        is_memory = memory_read | memory_write
        
        # This `is_memory` hack is to evade rust's overflow check.
        addr = (result.bitcast(UInt(32)) - is_memory.select(offset_reg[0].bitcast(UInt(32)), UInt(32)(0))).bitcast(Bits(32))
        request_addr = is_memory.select(addr[2:2+depth_log-1].bitcast(UInt(depth_log)), UInt(depth_log)(0))
        with Condition(memory_read):
            log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)
        with Condition(memory_write):
            log("mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} | wdada: 0x{:08x}", result, request_addr, a, b)
        
        with Condition(is_memory):  
            scoreboard['is_memory_read'][sb_index] = memory_read
            scoreboard['result'][sb_index] = signals.link_pc.select(pc0, result) 
            scoreboard['csr_id'][sb_index] = csr_id
            scoreboard['csr_new'][sb_index] = csr_new 
            scoreboard['sb_status'][sb_index] = Bits(2)(2)
 
        dcache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        dcache.name = 'dcache'
        dcache.build(we=memory_write, re=memory_read, wdata=b, addr=request_addr, user=memory)
        bound = dcache.bound.bind(rd=rd,index=sb_index )
        
        bound.async_called()
        wb = writeback.bind()
        with Condition(rd != Bits(5)(0)):
            log("own x{:02}          |", rd)

        
        return    br_dest, wb,  ex_update,execution_index,ex_data,predict_wrong





class Decoder(Module):
    
    def __init__(self):
        super().__init__(ports={
            'rdata': Port(Bits(32)),
            'fetch_addr': Port(Bits(32)),
        })
        self.name = 'D'
        
    @module.combinational
    def build(self, sb_tail:Array   ):
        
        inst = self.rdata.peek()
        fetch_addr = self.fetch_addr.peek()

        log("raw: 0x{:08x}  | addr: 0x{:05x} |", inst, fetch_addr)
        
        signals = decode_logic(inst)
        
        is_ebreak= (signals.rs1_valid & signals.imm_valid & ((signals.imm == Bits(32)(1))|(signals.imm == Bits(32)(0))) & (signals.alu == Bits(16)(0)))
        
        is_nop = ((inst == Bits(32)(51)) | (inst==Bits(32)(0))).select(Bits(1)(1),Bits(1)(0))
        
        Index = sb_tail[0] 
        inst, fetch_addr = self.pop_all_ports(False)
 
        
        with Condition(~is_nop):
            decode_signals = signals.value()
            decode_index = Index
            decode_fetch_addr = fetch_addr
            is_br =  signals.is_branch
            is_jalr = (signals.is_branch & (~signals.is_offset_br)&signals.link_pc)
            
            with Condition( (signals.rd_valid)&(signals.rd!=Bits(5)(0)) ): 
                rmt_update_rd = signals.rd
                rmt_update_index = Index
            with Condition(is_br & (~is_ebreak)):
                predicted_addr =( (signals.imm).bitcast(Int(32)) + fetch_addr.bitcast(Int(32)) ).bitcast(Bits(32))
                
        return  is_nop,rmt_update_rd,rmt_update_index,decode_index,decode_fetch_addr,decode_signals,predicted_addr,is_jalr 
    
class Fetcher(Module):
    
    def __init__(self):
        super().__init__(ports={})
        self.name = 'F'

    @module.combinational
    def build(self):
        
        pc_reg = RegArray(Bits(32), 1)
        addr = pc_reg[0]
        cycle_activate = (addr == Bits(32)(0)).select(Bits(1)(1),Bits(1)(0))
        return pc_reg, addr,cycle_activate
 
             
class Dispatch(Downstream):

    def __init__(self):
        super().__init__()
        self.name = 'p'

    @downstream.combinational
    def build(self,
            scoreboard:Array,
            executor:Module, 
            trigger:Value, 
            predict_wrong:Value,
            ex_bypass: Value, 
            pc_reg: Value,
            pc_addr: Value,
            decoder: Decoder,
            data: str,
            depth_log: int, 
            sb_head:Array,
            sb_tail:Array,
            predicted_addr:Value, 
            is_jal:Value,
            mem:Value,
            ex:Value, 
            RMT: Array ,
            execution_index:Value , 
            is_nop:Value, 
            rmt_update_rd:Value,
            rmt_update_index:Value,
            rmt_clear_rd:Value,
            rmt_clear_index:Value,
            ex_data:Value,
            mdata:Value,
            reg_file:Array,
            cur_index:Value,
            fetch_addr:Value,
            d_signals:Value, 
            m_index:Value,
            m_arg:Value ):
        
        trigger = trigger.optional(Bits(1)(0))
        
        br_signal = RegArray(UInt(32), 1  ) 
        br_flag = br_signal[0] 
        with Condition(br_flag<UInt(32)(1)):
            br_signal[0] = (predict_wrong.valid() | predicted_addr.valid()).select(UInt(32)(0), br_flag+UInt(32)(1) )
        

        predict_wrong = predict_wrong.optional(Bits(1)(0))
        
        #Fetch Impl  
        next_index2 = ( sb_tail[0].bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1) \
                      ).bitcast(Bits(SCOREBOARD.Bit_size)) & (Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size - 1))
         
        is_not_full_scoreboard = ( (next_index2 != sb_head[0])) | (~scoreboard['sb_valid'][sb_head[0]]) 
        is_jal = is_jal.optional(Bits(1)(0))
        should_fetch =  is_not_full_scoreboard & (~is_jal) 
        
        to_fetch = predicted_addr.optional(pc_addr)
        ex_bypass = ex_bypass.optional(to_fetch) 
        to_fetch = predict_wrong.select(ex_bypass,to_fetch) 
        icache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        icache.name = 'icache'
         
        real_fetch = should_fetch  

        icache.build(Bits(1)(0), real_fetch, to_fetch[2:2+depth_log-1].bitcast(Int(depth_log)), Bits(32)(0), decoder)
        
        with Condition(real_fetch):
            icache.bound.async_called(fetch_addr=to_fetch)
            pc_reg[0] = (to_fetch.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
            
        with Condition(~real_fetch):
            pc_reg[0] = to_fetch


        #update RMT and s
        
         
        is_nop = is_nop.optional(Bits(1)(0)) 
        update_tail =  ((~cur_index.valid()) ).select(
            sb_tail[0],
            ((
                (sb_tail[0].bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1))   
            ).bitcast(Bits(SCOREBOARD.Bit_size))) & (Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size - 1))  
        )

        bypass_tail =  (
            (
                (execution_index.optional(sb_tail[0])).bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1) 
            ).bitcast(Bits(SCOREBOARD.Bit_size)) & (Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size - 1))  
        )
        
        sb_tail[0] = predict_wrong.select( bypass_tail ,update_tail )
         
        rmt_clear_rd = rmt_clear_rd.optional(Bits(5)(0))
        rmt_up_rd = rmt_update_rd.optional(Bits(5)(0))
        mem_index = mem.optional(NoDep)
        ex_index = ex.optional(NoDep)  
        rmt_cl_index = rmt_clear_index.optional(NoDep)
         
        cur_index = cur_index.optional(NoDep)
        Fetch_addr = fetch_addr.optional(Bits(32)(0))
        de_signals = decoder_signals.view(d_signals.optional(Bits(97)(0)))
        e_data = ex_data.optional(Bits(32)(0))
        m_data = mdata.optional(Bits(32)(0))
        mem_valid = m_arg.valid()
        m_index = m_index.optional(NoDep)
        m_arg = m_arg.optional(Bits(32)(0))
         
        with Condition(predict_wrong): 
            with Condition(br_flag!=UInt(32)(0)):
                for i in range(SCOREBOARD.size): 
                    move1 = (Bits(SCOREBOARD.Bit_size)(i) <sb_tail[0]) & (Bits(SCOREBOARD.Bit_size)(i) >= bypass_tail)
    
                    move2 = (Bits(SCOREBOARD.Bit_size)(i) >=bypass_tail) & ( (sb_tail[0]<bypass_tail)  )
                    move3 = ( (sb_tail[0]<bypass_tail) & (Bits(SCOREBOARD.Bit_size)(i) <sb_tail[0]) )
                    with Condition( (move1 | move2 | move3) ):
                        scoreboard['sb_valid'][i] = Bits(1)(0)
                        scoreboard['sb_status'][i] = Bits(2)(0)
                         
            with Condition(mem_valid):
                move1 = (m_index <sb_tail[0]) & (m_index >= bypass_tail)
                move2 = (m_index >=bypass_tail) & ( (sb_tail[0]<bypass_tail)  )
                move3 = ( (sb_tail[0]<bypass_tail) & (m_index <sb_tail[0]) )
                un_pw = ~(move1 | move2 | move3)
                # log("un_pw {}  m_index {} execution_index {} ",un_pw,m_index,execution_index)
                with Condition(un_pw): 
                    scoreboard['mdata'][m_index] = m_arg
                    scoreboard['sb_status'][m_index] = Bits(2)(3)
                    
            with Condition( (rmt_clear_rd != Bits(5)(0)) & (RMT[rmt_clear_rd]==rmt_cl_index)):
                RMT[rmt_clear_rd] = NoDep

        with Condition(~predict_wrong):
            RMT[rmt_up_rd] =  (rmt_up_rd == Bits(5)(0)).select( NoDep ,rmt_update_index )   
            
            with Condition( (rmt_clear_rd != Bits(5)(0))& (rmt_clear_rd!=rmt_up_rd)& (RMT[rmt_clear_rd]==rmt_cl_index)):
                RMT[rmt_clear_rd] = NoDep 

            with Condition(mem_valid):
                scoreboard['mdata'][m_index] = m_arg
                scoreboard['sb_status'][m_index] = Bits(2)(3)
          
            for i in range(SCOREBOARD.size):
                with Condition(scoreboard['sb_valid'][i] & (scoreboard['sb_status'][i] == Bits(2)(0))):
                    rs1_dep = scoreboard['rs1_dep'][i]
                    rs2_dep = scoreboard['rs2_dep'][i]

                    mem1 = (rs1_dep == mem_index)
                    mem2 = (rs2_dep == mem_index)
                    rs1_prefetch = (~scoreboard['rs1_ready'][i]) & (mem1 | (rs1_dep == ex_index)) & (scoreboard['sb_status'][rs1_dep] != Bits(2)(3))
                    rs2_prefetch = (~scoreboard['rs2_ready'][i]) & (mem2| (rs2_dep == ex_index)) & (scoreboard['sb_status'][rs2_dep] != Bits(2)(3))
                    rs1_update = (~scoreboard['rs1_ready'][i]) & (scoreboard['sb_status'][rs1_dep] == Bits(2)(3))
                    rs2_update = (~scoreboard['rs2_ready'][i]) & (scoreboard['sb_status'][rs2_dep] == Bits(2)(3))

                    with Condition(rs1_prefetch): 
                        scoreboard['rs1_value'][i] =  (mem1).select(m_data, e_data)  
                        scoreboard['rs1_ready'][i] = Bits(1)(1)

                    with Condition(rs2_prefetch): 
                        scoreboard['rs2_value'][i] = (mem2).select(m_data, e_data)    
                        scoreboard['rs2_ready'][i] = Bits(1)(1)

                    with Condition(rs1_update):
                        rs1_update_value = (scoreboard['is_memory_read'][rs1_dep]).select(scoreboard['mdata'][rs1_dep], scoreboard['result'][rs1_dep])
                        scoreboard['rs1_value'][i] = rs1_update_value  
                        scoreboard['rs1_ready'][i] = Bits(1)(1)

                    with Condition(rs2_update):
                        rs2_update_value = (scoreboard['is_memory_read'][rs2_dep]).select(scoreboard['mdata'][rs2_dep], scoreboard['result'][rs2_dep])
                        scoreboard['rs2_value'][i] = rs2_update_value   
                        scoreboard['rs2_ready'][i] = Bits(1)(1)
         
            #Dispatch 
 
            valid_temp = Bits(1)(0)
             
            dispatch_index = Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
            branch_index = Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
            br_valid = Bits(1)(0) 
            for i in range(SCOREBOARD.size):  
                signals =  scoreboard['signals'][i]
                       
                valid_temp = (scoreboard['sb_valid'][i] & 
                            (scoreboard['sb_status'][i] == Bits(2)(0)) & 
                            scoreboard['rs1_ready'][i] & 
                            scoreboard['rs2_ready'][i])
                
                dispatch_index = valid_temp.select(Bits(SCOREBOARD.Bit_size)(i), dispatch_index)
                branch_index = ((~br_valid)&(valid_temp&signals.is_branch )).select(Bits(SCOREBOARD.Bit_size)(i),branch_index )
                
                br_valid = (valid_temp & signals.is_branch ) | br_valid
                 
            d_id = (br_valid).select(branch_index,dispatch_index)
             
            valid_global =  (d_id== NoDep).select(Bits(1)(0),Bits(1)(1)) 
              
            with Condition(valid_global ): 
                scoreboard['sb_status'][d_id] = Bits(2)(1) 
                  
                call = executor.async_called(
                    sb_index=d_id
                )

                call.bind.set_fifo_depth()
        

            with Condition( ~is_nop & (cur_index!=NoDep)): 
               
                newest_index = cur_index     
                newest_entry = add_entry(de_signals,scoreboard,RMT,reg_file,Fetch_addr,mem_index,ex_index,e_data,m_data)
                
                is_ebreak = (signals.rs1_valid & signals.imm_valid & ((signals.imm == Bits(32)(1))|(signals.imm == Bits(32)(0)))\
                            & (signals.alu == Bits(16)(0))).select(Bits(1)(1),Bits(1)(0))

                early_dispatch_valid =( newest_entry.rs1_ready & newest_entry.rs2_ready &(~is_ebreak)).select(Bits(1)(1),Bits(1)(0))
                
                exe_dispatch_valid = early_dispatch_valid &(~valid_global)
                scoreboard['sb_valid'][newest_index] = Bits(1)(1) 
                scoreboard['rs1_ready'][newest_index] = newest_entry.rs1_ready
                scoreboard['rs2_ready'][newest_index] = newest_entry.rs2_ready
                scoreboard['rs1_value'][newest_index] = newest_entry.rs1_value
                scoreboard['rs2_value'][newest_index] = newest_entry.rs2_value
                scoreboard['rs1_dep'][newest_index] = newest_entry.rs1_dep
                scoreboard['rs2_dep'][newest_index] =  newest_entry.rs2_dep
                scoreboard['signals'][newest_index] =  newest_entry.signals
                scoreboard['fetch_addr'][newest_index] = newest_entry.fetch_addr
                 
                with Condition(exe_dispatch_valid ):
                     
                    scoreboard['sb_status'][newest_index] = Bits(2)(1)
                    
                    call = executor.async_called( sb_index=newest_index)
                    
                    call.bind.set_fifo_depth()  


class MemUser(Module):
    def __init__(self, width):
        super().__init__(
            ports={'rdata': Port(Bits(width))}, 
        )
    @module.combinational
    def build(self):
        width = self.rdata.dtype.bits
        rdata = self.pop_all_ports(False)
        rdata = rdata.bitcast(Int(width))
        offset_reg = RegArray(Bits(width), 1)
        offset_reg[0] = rdata.bitcast(Bits(width))
        return offset_reg
 
class Driver(Module):
    
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, fetcher: Module, user: Module):
        init_reg = RegArray(UInt(1), 1, initializer=[1])
        init_cache = SRAM(width=32, depth=32, init_file=f"{workspace}/workload.init")
        init_cache.name = 'init_cache'
        init_cache.build(we=Bits(1)(0), re=init_reg[0].bitcast(Bits(1)), wdata=Bits(32)(0), addr=Bits(5)(0), user=user)
        # Initialze offset at first cycle
        with Condition(init_reg[0]==UInt(1)(1)):
            init_cache.bound.async_called()
            init_reg[0] = UInt(1)(0)
        # Async_call after first cycle
        with Condition(init_reg[0] == UInt(1)(0)):
            
            d_call = fetcher.async_called()
         

def build_cpu(depth_log):
    sys = SysBuilder('o3_cpu')

    with sys:

        # Data Types 
        bits32  = Bits(32)

        user = MemUser(32)
        offset_reg = user.build()

        fetcher = Fetcher()
        pc_reg, pc_addr ,cycle_activate= fetcher.build()
 

        # Data Structures
        reg_file    = RegArray(bits32, 32)

        reg_map_table = RegArray(Bits(SCOREBOARD.Bit_size),32,initializer=[SCOREBOARD.size]*32,attr=[Array.FULLY_PARTITIONED])

    
        scoreboard = {
            'sb_valid': RegArray(Bits(1), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size,attr=[Array.FULLY_PARTITIONED]),
            'rs1_ready': RegArray(Bits(1), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size,attr=[Array.FULLY_PARTITIONED]),
            'rs2_ready': RegArray(Bits(1), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size,attr=[Array.FULLY_PARTITIONED]),
            'rs1_value': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size,attr=[Array.FULLY_PARTITIONED]),
            'rs2_value': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size,attr=[Array.FULLY_PARTITIONED]),
            'rs1_dep': RegArray(Bits(SCOREBOARD.Bit_size), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'rs2_dep': RegArray(Bits(SCOREBOARD.Bit_size), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'result': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'sb_status': RegArray(Bits(2), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size,attr=[Array.FULLY_PARTITIONED]),
            'signals': RegArray(decoder_signals, SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'fetch_addr': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'is_memory_read': RegArray(Bits(1), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'mdata': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'csr_id': RegArray(Bits(4), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'csr_new': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
        }

  
        sb_head = RegArray(Bits(SCOREBOARD.Bit_size), 1, initializer=[0])
        sb_tail = RegArray(Bits(SCOREBOARD.Bit_size), 1, initializer=[0])


        csr_file = RegArray(Bits(32), 16, initializer=[0]*16)


        writeback = WriteBack()
        rmt_clear_rd,rmt_clear_index= writeback.build(reg_file = reg_file , csr_file = csr_file,sb_valid_array=scoreboard['sb_valid'],sb_status_array=scoreboard['sb_status'],sb_head=sb_head, \
                    is_memory_read_array = scoreboard['is_memory_read'] , result_array = scoreboard['result']  , \
                    mdata_array = scoreboard['mdata'] , \
                    csr_id_array = scoreboard['csr_id'] , csr_new_array = scoreboard['csr_new'] , \
                    signals_array = scoreboard['signals']  )
 
        memory_access = MemoryAccess()

        executor = Execution()
        
        
        ex_bypass, wb,  ex_update,execution_index,ex_data,predict_wrong = executor.build( 
            rf = reg_file,
            csr_f = csr_file,
            memory = memory_access,
            writeback = writeback,
            data = f'{workspace}/workload.data',
            depth_log = depth_log,
            scoreboard=scoreboard, 
            offset_reg = offset_reg,
            )
        
        
        mem_update,m_data,m_arg,m_index = memory_access.build(
            writeback = wb, 
            scoreboard=scoreboard, 
        )
        
        
        decoder = Decoder()
        
        dispatch = Dispatch()
        
             
         
        
        is_nop,  rmt_update_rd,rmt_update_index,decode_index,decode_fetch_addr,decode_signals,predicted_addr,is_jal= decoder.build( sb_tail=sb_tail )

        dispatch.build(executor=executor,scoreboard=scoreboard,trigger=cycle_activate, \
             predict_wrong=predict_wrong ,ex_bypass = ex_bypass, pc_reg = pc_reg, pc_addr =pc_addr, decoder =decoder, data=f'{workspace}/workload.exe',depth_log= depth_log, \
                            sb_head = sb_head, sb_tail=sb_tail,predicted_addr = predicted_addr,is_jal =is_jal , \
            ex=ex_update,mem=mem_update, RMT=reg_map_table,execution_index=execution_index , \
            rmt_clear_rd=rmt_clear_rd,rmt_clear_index=rmt_clear_index,\
                rmt_update_rd=rmt_update_rd,is_nop=is_nop,rmt_update_index=rmt_update_index,mdata=m_data,ex_data = ex_data,reg_file=reg_file,\
                      cur_index=decode_index, fetch_addr=decode_fetch_addr,d_signals=decode_signals, \
                        m_index=m_index,m_arg=m_arg )
         
        
        driver = Driver()
        driver.build(fetcher, user)
 

    print(sys)
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=600000,
        idle_threshold=600000,
        resource_base='',
        fifo_depth=1,
    )

    simulator_path, verilog_path = elaborate(sys, **conf)

    return sys, simulator_path, verilog_path



def run_cpu(sys, simulator_path, verilog_path, workload='default'):
    with sys:
        with open(f'{workspace}/workload.config') as f:
            raw = f.readline()
            raw = raw.replace('offset:', "'offset':").replace('data_offset:', "'data_offset':")
            offsets = eval(raw)
            value = hex(offsets['data_offset'])
            value = value[1:] if value[0] == '-' else value
            value = value[2:]
            open(f'{workspace}/workload.init', 'w').write(value)

    report = True

    if report:
        raw = utils.run_simulator(simulator_path, False)
        open(f'{workload}.log', 'w').write(raw) 
        raw = utils.run_verilator(verilog_path, False)
        open(f'{workload}.verilog.log', 'w').write(raw)
    else:
        raw = utils.run_simulator(simulator_path)
        open('raw.log', 'w').write(raw)
        check()
        raw = utils.run_verilator(verilog_path)
        open('raw.log', 'w').write(raw)
        check()
        os.remove('raw.log')


def check():

    script = f'{workspace}/workload.sh'
    if os.path.exists(script):
        res = subprocess.run([script, 'raw.log', f'{workspace}/workload.data'])
         
    else:
        script = f'{current_path}/../utils/find_pass.sh'
       
        res = subprocess.run([script, 'raw.log'])
    assert res.returncode == 0, f'Failed test: {res.returncode}'
    print('Test passed!!!')

 
def cp_if_exists(src, dst, placeholder):
    if os.path.exists(src):
        shutil.copy(src, dst)
    elif placeholder:
        open(dst, 'w').write('')

def init_workspace(base_path, case):
    if os.path.exists(f'{workspace}'):
        shutil.rmtree(f'{workspace}')
    os.mkdir(f'{workspace}')
    cp_if_exists(f'{base_path}/{case}.exe', f'{workspace}/workload.exe', False)
    cp_if_exists(f'{base_path}/{case}.data', f'{workspace}/workload.data', True)
    cp_if_exists(f'{base_path}/{case}.config', f'{workspace}/workload.config', False)
    cp_if_exists(f'{base_path}/{case}.sh', f'{workspace}/workload.sh', False)


if __name__ == '__main__':
    # Build the CPU Module only once
    sys, simulator_path, verilog_path = build_cpu(depth_log=16)
    print("o3-CPU built successfully!")
    # Define workloads
    wl_path = f'{utils.repo_path()}/examples/minor-cpu/workloads'
    workloads = [
        '0to100',
        # 'multiply', 
        #'dhrystone',
        'median',
        'multiply',
        'qsort',  
        'rsort',
        'towers',
        'vvadd',
    ]
    # Iterate workloads
    for wl in workloads:
        # Copy workloads to tmp directory and rename to workload.
        init_workspace(wl_path, wl)
        run_cpu(sys, simulator_path, verilog_path , wl)
    print("o3-CPU workloads ran successfully!")

    #================================================================================================
    # The same logic should be able to apply to the tests below, while the offsets&data_offsets should be changed accordingly.
    # Define test cases
    test_cases = [
        #'rv32ui-p-add',
        #'rv32ui-p-addi',
        #'rv32ui-p-and',
        #'rv32ui-p-andi',
        #'rv32ui-p-auipc',
        #'rv32ui-p-beq',
        #'rv32ui-p-bge',
        #'rv32ui-p-bgeu',
        #'rv32ui-p-blt',
        #'rv32ui-p-bltu',
        #'rv32ui-p-bne',
        #'rv32ui-p-jal',
        #'rv32ui-p-jalr',
        #'rv32ui-p-lui',
        #'rv32ui-p-lw',
        #'rv32ui-p-or',
        #'rv32ui-p-ori',
        #'rv32ui-p-sll',
        #'rv32ui-p-slli',
        #'rv32ui-p-sltu',
        #'rv32ui-p-srai',
        #'rv32ui-p-srl',
        #'rv32ui-p-srli',
        #'rv32ui-p-sub',
        #'rv32ui-p-sw',
        #'rv32ui-p-xori',
        #'rv32ui-p-lbu',#TO DEBUG&TO CHECK
        #'rv32ui-p-sb',#TO CHECK
    ]
    tests = f'{utils.repo_path()}/examples/minor-cpu/unit-tests'
    # Iterate test cases
    for case in test_cases:
        # Copy test cases to tmp directory and rename to workload.
        init_workspace(tests, case)
        run_cpu(sys, simulator_path, verilog_path)
    print("o3-CPU tests ran successfully!")
