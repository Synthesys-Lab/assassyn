'''  
'''

import os
import shutil
import argparse

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
                'sb_index':Port(Bits(SCOREBOARD.Bit_size)) ,
                 
            })
        self.name = "E"

    @module.combinational
    def build(
        self,  
        rf: Array, 
        csr_f: Array,
        memory: Module,  
        data: str,
        depth_log: int,
        scoreboard:Array, 
        signals_array:Array,
        offset_reg: Array, 
        ):

       
        sb_index = self.sb_index.pop()
        
        signals=signals_array[sb_index]
        fetch_addr=scoreboard['fetch_addr'][sb_index]

        rs1 = signals.rs1 
        rs2 = signals.rs2
        rd = signals.rd

        rs1_dep = scoreboard['rs1_dep'][sb_index]
 
        rs1_dep_valid  = ( (scoreboard['sb_status'][rs1_dep]!=Bits(2)(1)) & scoreboard['sb_valid'][rs1_dep]   )&(signals_array[rs1_dep].rd == rs1)

        rs1_dep_result = (signals_array[rs1_dep].memory[0:0]).select( scoreboard['mdata'][rs1_dep], scoreboard['result'][rs1_dep])
        rs1_value=rs1_dep_valid.select(rs1_dep_result ,rf[rs1]) 

        rs2_dep = scoreboard['rs2_dep'][sb_index] 

        rs2_dep_valid  = ((scoreboard['sb_status'][rs2_dep]!=Bits(2)(1)) & scoreboard['sb_valid'][rs2_dep]  )&(signals_array[rs2_dep].rd == rs2)

        rs2_dep_result = (signals_array[rs2_dep].memory[0:0]).select( scoreboard['mdata'][rs2_dep], scoreboard['result'][rs2_dep])
        rs2_value=rs2_dep_valid.select(rs2_dep_result,rf[rs2] )
 
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
  
        pc0 = (fetch_addr.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
        
               
        is_memory = memory_read | memory_write
         
        addr = (result.bitcast(UInt(32)) - is_memory.select(offset_reg[0].bitcast(UInt(32)), UInt(32)(0))).bitcast(Bits(32))
        request_addr = is_memory.select(addr[2:2+depth_log-1].bitcast(UInt(depth_log)), UInt(depth_log)(0))
        with Condition(memory_read):
            log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)
        with Condition(memory_write):
            log("mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} | wdada: 0x{:08x}", result, request_addr, a, b)
         
              
        dcache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        dcache.name = 'dcache'
        dcache.build(we=memory_write, re=memory_read, wdata=b, addr=request_addr)
        bound = memory.bind( index=sb_index )
        
        bound.async_called() 

        with Condition(signals.csr_write):
            csr_f[csr_id] = csr_new
 
          
        with Condition(signals.is_branch):
            br_dest = condition[0:0].select(result, pc0)
            execution_index = sb_index 
            log("condition: {}.a.b | a: {:08x}  | b: {:08x}   |", condition[0:0], result, pc0)
            predict_wrong = condition[0:0].select(Bits(1)(0),Bits(1)(1)) 
            predict_wrong = (signals.is_branch & (~signals.is_offset_br)&signals.link_pc).select(Bits(1)(1),predict_wrong) 
           
        with Condition(~is_memory ): 
            with Condition((rd != Bits(5)(0))):
                exe_update = sb_index   
                ex_data = signals.link_pc.select(pc0, result) 
                 
                 
        
        return    br_dest,  exe_update,execution_index,ex_data,predict_wrong





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
         
        Index = sb_tail[0] 
        inst, fetch_addr = self.pop_all_ports(False)
  
        decode_signals = signals.value()
        decode_index = Index
        decode_fetch_addr = fetch_addr
        is_br =  signals.is_branch
        is_jalr = (is_br & (~signals.is_offset_br)&signals.link_pc)
        new_rd = signals.rd
        with Condition( new_rd !=Bits(5)(0)): 
            rmt_update_rd = new_rd
        with Condition(is_br  ):
            predicted_addr =( (signals.imm).bitcast(Int(32)) + fetch_addr.bitcast(Int(32)) ).bitcast(Bits(32))
                
        return  rmt_update_rd,decode_index,decode_fetch_addr,decode_signals,predicted_addr,is_jalr 
    
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
            exe_pass_id:Value, 
            RMT: Array ,
            execution_index:Value ,  
            rmt_update_rd:Value, 
            rmt_clear_rd:Value,
            rmt_clear_index:Value,
            ex_data:Value, 
            signals_array:Array,
            cur_index:Value,
            fetch_addr:Value,
            d_signals:Value, 
            m_index:Value, 
            writeback:Module):
        
        trigger = trigger.optional(Bits(1)(0))
        
        br_signal = RegArray(UInt(32), 1  ) 
        br_flag = br_signal[0] 
        with Condition(br_flag<UInt(32)(1)):
            br_signal[0] = (predict_wrong.valid() | predicted_addr.valid()).select(UInt(32)(0), br_flag+UInt(32)(1) )
         
        predict_wrong = predict_wrong.optional(Bits(1)(0))
        
        stail = sb_tail[0]
        #Fetch Impl  
        next_index2 =   (stail.bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1)).bitcast(Bits(SCOREBOARD.Bit_size))
        next_index2 = (next_index2==NoDep).select(Bits(SCOREBOARD.Bit_size)(0),next_index2)
        
        shead = sb_head[0]
        is_not_full_scoreboard = ( (next_index2 != shead)) | (~scoreboard['sb_valid'][shead]) 
        is_jal = is_jal.optional(Bits(1)(0))
        real_fetch =  is_not_full_scoreboard & (~is_jal) 
        
        to_fetch = predicted_addr.optional(pc_addr)
        ex_bypass = ex_bypass.optional(to_fetch) 
        to_fetch = predict_wrong.select(ex_bypass,to_fetch) 
        icache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        icache.name = 'icache'
          
        icache.build(Bits(1)(0), real_fetch, to_fetch[2:2+depth_log-1].bitcast(Int(depth_log)), Bits(32)(0))
        
        with Condition(real_fetch):
            decoder.async_called(fetch_addr=to_fetch)
            pc_reg[0] = (to_fetch.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
            
        with Condition(~real_fetch):
            pc_reg[0] = to_fetch


        #update RMT and register files
          
        update_tail =  ((~cur_index.valid()) ).select( stail , \
                (stail.bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1)).bitcast(Bits(SCOREBOARD.Bit_size)) )  
          
        
        update_tail = (update_tail==NoDep).select(Bits(SCOREBOARD.Bit_size)(0),update_tail)

        bypass_tail =  (
            (
                (execution_index.optional(stail)).bitcast(Int(SCOREBOARD.Bit_size)) + Int(SCOREBOARD.Bit_size)(1) 
            ).bitcast(Bits(SCOREBOARD.Bit_size)) 
        )
        bypass_tail = (bypass_tail==NoDep).select(Bits(SCOREBOARD.Bit_size)(0),bypass_tail)
        
        sb_tail[0] = predict_wrong.select( bypass_tail ,update_tail ) 
        rmt_clear_rd = rmt_clear_rd.optional(Bits(5)(0))
        rmt_up_rd = rmt_update_rd.optional(Bits(5)(0)) 
        rmt_cl_index = rmt_clear_index.optional(NoDep)

        exe_bypass = ex_data.valid() 
        
        exe_pass_index = exe_bypass.select(exe_pass_id,NoDep)  
        with Condition(exe_bypass):
            scoreboard['result'][execution_index] = ex_data
            scoreboard['sb_status'][execution_index] = Bits(2)(2) 
         
        newest_index = cur_index.optional(NoDep)
        Fetch_addr = fetch_addr.optional(Bits(32)(0))
        de_signals = decoder_signals.view(d_signals.optional(Bits(97)(0)))
          

        mem_pass_index = m_index.optional(NoDep)
         
         
        writeback.async_called()
 
        rmt_clear =  (rmt_clear_rd != Bits(5)(0)) & (RMT[rmt_clear_rd]==rmt_cl_index)
        with Condition(predict_wrong): 
            with Condition(br_flag!=UInt(32)(0)):
                for i in range(SCOREBOARD.size): 
                    move1 = (Bits(SCOREBOARD.Bit_size)(i) <stail) & (Bits(SCOREBOARD.Bit_size)(i) >= bypass_tail)
    
                    move2 = (Bits(SCOREBOARD.Bit_size)(i) >=bypass_tail) & ( (stail<bypass_tail)  )
                    move3 = ( (stail<bypass_tail) & (Bits(SCOREBOARD.Bit_size)(i) <stail) )
                    with Condition( (move1 | move2 | move3) ): 
                        scoreboard['sb_valid'][i] = Bits(1)(0) 
                  
            with Condition(rmt_clear):
                RMT[rmt_clear_rd] = NoDep


        with Condition(~predict_wrong):
            RMT[rmt_up_rd] =  (rmt_up_rd == Bits(5)(0)).select( NoDep ,newest_index )   
            
            with Condition( rmt_clear & (rmt_clear_rd!=rmt_up_rd) ):
                RMT[rmt_clear_rd] = NoDep  
            #Dispatch 
            valid_temp = Bits(1)(0)
            
            dispatch_index = Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
            branch_index = Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
            br_valid = Bits(1)(0) 
            valid_global = Bits(1)(0)  
            for i in range(SCOREBOARD.size):   
                rs1_dep = scoreboard['rs1_dep'][i]
                mem1 = (rs1_dep == mem_pass_index)
                rs1_dep_rd = (scoreboard['sb_status'][rs1_dep]==Bits(2)(3))
                rs1_prefetch =   (mem1 | (rs1_dep == exe_pass_index)) |  rs1_dep_rd

                rs2_dep = scoreboard['rs2_dep'][i]
                mem2 = (rs2_dep == mem_pass_index)
                rs2_dep_rd = (scoreboard['sb_status'][rs2_dep]==Bits(2)(3))
                rs2_prefetch =   (mem2| (rs2_dep == exe_pass_index))  | rs2_dep_rd
                
                to_issue = scoreboard['sb_valid'][i] & (scoreboard['sb_status'][i] == Bits(2)(0))
                with Condition(to_issue): 
                    with Condition(rs1_prefetch):   
                            scoreboard['rs1_ready'][i] = Bits(1)(1)
 
                    with Condition(rs2_prefetch):    
                            scoreboard['rs2_ready'][i] = Bits(1)(1)
  
                valid_temp = (to_issue & 
                            (scoreboard['rs1_ready'][i] | rs1_prefetch) & 
                            (scoreboard['rs2_ready'][i] | rs2_prefetch) )
                
                is_br =  ((signals_array[i].is_branch )& valid_temp)
                       
                dispatch_index = valid_temp.select(Bits(SCOREBOARD.Bit_size)(i), dispatch_index) 
                branch_index = (is_br&(~br_valid)).select(Bits(SCOREBOARD.Bit_size)(i),branch_index)
                br_valid =  is_br | br_valid 
                valid_global = valid_global | valid_temp
                 
                 
            d_id = (br_valid).select(branch_index,dispatch_index)
             
            with Condition(valid_global ): 
                scoreboard['sb_status'][d_id] = Bits(2)(1) 
                
                call = executor.async_called(
                    sb_index=d_id 
                )

                call.bind.set_fifo_depth()
        

            with Condition(newest_index!=NoDep): 
                
                newest_index = cur_index     
                rs1 = de_signals.rs1
                rs2 = de_signals.rs2
                rs1_valid = de_signals.rs1_valid
                rs2_valid = de_signals.rs2_valid

                entry_rs1 = RMT[rs1] 
                entry_rs2 = RMT[rs2]
 
                rs1_ready,rs2_ready = call_rs(rs1,rs2,rs1_valid ,rs2_valid ,scoreboard,signals_array,entry_rs1,entry_rs2,mem_pass_index,exe_pass_index )
                  
                exe_dispatch_valid =  (~valid_global)&(rs1_ready & rs2_ready )
                scoreboard['sb_valid'][newest_index] = Bits(1)(1) 
                
                scoreboard['rs1_ready'][newest_index] = rs1_ready
                scoreboard['rs2_ready'][newest_index] = rs2_ready
 
                scoreboard['rs1_dep'][newest_index] =  entry_rs1
                scoreboard['rs2_dep'][newest_index] =  entry_rs2
                signals_array[newest_index] = de_signals
                scoreboard['fetch_addr'][newest_index] =  Fetch_addr
                scoreboard['sb_status'][newest_index] = exe_dispatch_valid.select( Bits(2)(1), Bits(2)(0))

                with Condition(exe_dispatch_valid ):  
                    
                    call = executor.async_called( sb_index=newest_index )
                    
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
             
            'rs1_dep': RegArray(Bits(SCOREBOARD.Bit_size), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'rs2_dep': RegArray(Bits(SCOREBOARD.Bit_size), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'result': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ),
            'sb_status': RegArray(Bits(2), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size,attr=[Array.FULLY_PARTITIONED]),
            
            'fetch_addr': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ), 
            'mdata': RegArray(Bits(32), SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size ) 
        }

        signals_array = RegArray(decoder_signals, SCOREBOARD.init_size,initializer=[0]*SCOREBOARD.init_size )
        sb_head = RegArray(Bits(SCOREBOARD.Bit_size), 1, initializer=[0])
        sb_tail = RegArray(Bits(SCOREBOARD.Bit_size), 1, initializer=[0])


        csr_file = RegArray(Bits(32), 16, initializer=[0]*16)


        writeback = WriteBack()
        rmt_clear_rd,rmt_clear_index= writeback.build(reg_file = reg_file , scoreboard=scoreboard,sb_head=sb_head,  signals_array = signals_array )
 
        memory_access = MemoryAccess()

        executor = Execution()
        
        
        ex_bypass,   exe_update,execution_index,ex_data,predict_wrong = executor.build( 
            rf = reg_file,
            csr_f = csr_file,
            memory = memory_access, 
            data = f'{workspace}/workload.data',
            depth_log = depth_log,
            scoreboard=scoreboard, 
            offset_reg = offset_reg,
            signals_array=signals_array
            )
        
        
        m_index = memory_access.build( 
            scoreboard=scoreboard,  
        )
        
        
        decoder = Decoder()
        
        dispatch = Dispatch()
        
             
         
        
        rmt_update_rd,decode_index,decode_fetch_addr,decode_signals,predicted_addr,is_jal= decoder.build( sb_tail=sb_tail )

        dispatch.build(executor=executor,scoreboard=scoreboard,trigger=cycle_activate, \
             predict_wrong=predict_wrong ,ex_bypass = ex_bypass, pc_reg = pc_reg, pc_addr =pc_addr, decoder =decoder, data=f'{workspace}/workload.exe',depth_log= depth_log, \
                            sb_head = sb_head, sb_tail=sb_tail,predicted_addr = predicted_addr,is_jal =is_jal , \
            exe_pass_id=exe_update , RMT=reg_map_table,execution_index=execution_index , \
            rmt_clear_rd=rmt_clear_rd,rmt_clear_index=rmt_clear_index,\
                rmt_update_rd=rmt_update_rd, ex_data = ex_data, \
                      cur_index=decode_index, fetch_addr=decode_fetch_addr,d_signals=decode_signals, \
                        m_index=m_index,signals_array=signals_array,writeback = writeback )
         
        
        driver = Driver()
        driver.build(fetcher, user)

        '''RegArray exposing'''
        sys.expose_on_top(reg_file, kind='Output') 
        sys.expose_on_top(csr_file, kind='Inout')
        sys.expose_on_top(pc_reg, kind='Output')

         
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

    parser = argparse.ArgumentParser(description="Run CPU workloads and tests.")
 
    parser.add_argument("workloads", nargs="*", default=[],
                        help="List of workloads to run.  If none specified, run none.")
    parser.add_argument("-t", "--tests", nargs="*", default=[],
                        help="List of test cases to run. If none specified, run none.")
    parser.add_argument("--all-workloads", action="store_true",
                        help="Run all available workloads (overrides 'workloads' argument).")
    parser.add_argument("--all-tests", action="store_true",
                        help="Run all available tests (overrides 'tests' argument).")
 
    args = parser.parse_args()

    # Build the CPU Module only once
    sys, simulator_path, verilog_path = build_cpu(depth_log=16)
    print("o3-CPU built successfully!")
    # Define workloads
    wl_path = f'{utils.repo_path()}/examples/minor-cpu/workloads'
    workloads = [
        '0to100', 
        #'dhrystone',
        'median',
        'multiply',
        'qsort',  
        'rsort',
        'towers',
        'vvadd',
    ]

    test_cases = [
        'rv32ui-p-add',
        'rv32ui-p-addi',
        'rv32ui-p-and',
        'rv32ui-p-andi',
        'rv32ui-p-auipc',
        'rv32ui-p-beq',
        'rv32ui-p-bge',
        'rv32ui-p-bgeu',
        'rv32ui-p-blt',
        'rv32ui-p-bltu',
        'rv32ui-p-bne',
        'rv32ui-p-jal',
        'rv32ui-p-jalr',
        'rv32ui-p-lui',
        'rv32ui-p-lw',
        'rv32ui-p-or',
        'rv32ui-p-ori',
        'rv32ui-p-sll',
        'rv32ui-p-slli',
        'rv32ui-p-sltu',
        'rv32ui-p-srai',
        'rv32ui-p-srl',
        'rv32ui-p-srli',
        'rv32ui-p-sub',
        'rv32ui-p-sw',
        'rv32ui-p-xori',
        'rv32ui-p-lbu',#TO DEBUG&TO CHECK
        'rv32ui-p-sb',#TO CHECK
    ]
    tests = f'{utils.repo_path()}/examples/minor-cpu/unit-tests'
    
    if args.all_workloads:
        run_workloads = workloads
    elif args.workloads:
        run_workloads = []
        for wl in args.workloads:
            if wl in workloads:
                run_workloads.append(wl)
            else:
                print(f"Warning: Workload '{wl}' not found, skipping.")
    else:
         run_workloads = [] # Default: run no workloads if not specified

    for wl in run_workloads:
        init_workspace(wl_path, wl)
        run_cpu(sys, simulator_path, verilog_path, wl)
    if run_workloads:
        print("o3-CPU workloads ran successfully!")
 

    if args.all_tests:
        run_tests = test_cases
    elif args.tests:
        run_tests = []
        for test_case in args.tests:
            if test_case in test_cases:
                run_tests.append(test_case)
            else:
                print(f"Warning: Test case '{test_case}' not found, skipping.")
    else:
        run_tests = []  # Default, run no tests if not specified.
    
    for case in run_tests:
        # Copy test cases to tmp directory and rename to workload.
        init_workspace(tests, case)
        run_cpu(sys, simulator_path, verilog_path)
    if run_tests:
        print("o3-CPU tests ran successfully!")
