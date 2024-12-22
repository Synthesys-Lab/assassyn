'''  
'''
 

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

from opcodes import *
from decoder import *
from writeback import *
from memory_access import *
from scoreboard import *

offset = None
data_offset = None



class Execution(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'rs1_value':Port(Bits(32)),
                'rs2_value':Port(Bits(32)),
                'signals': Port(deocder_signals),
                'fetch_addr': Port(Bits(32)),
                'sb_index':Port(Bits(SCOREBOARD.Bit_size))
                
            })
        self.name = "E"

    @module.combinational
    def build(
        self, 
        pc: Array,
        rf: Array, 
        csr_f: Array,
        memory: Module, 
        writeback: Module,
        data: str,
        depth_log: int,
        scoreboard:Array,
        RMT:Array 
        ):

        csr_id = Bits(4)(0)
        
        signals = self.signals.pop()
        rs1_value = self.rs1_value.pop()
        rs2_value = self.rs2_value.pop()
        fetch_addr = self.fetch_addr.pop()
        sb_index = self.sb_index.pop()
        # rs1_value,rs2_value,signals, fetch_addr = self.pop_all_ports(False)

        rs1 = signals.rs1
        rs2 = signals.rs2
        rd = signals.rd

        predict_wrong = RegArray(Bits(1),1,initializer=[0]) 

        ex_valid = Bits(1)(1)&Bits(1)(1)
        
        

        raw_id = [
        (773, 1), #mtvec
        (833,2), #mepc
        (772, 4), #mie
        (768,8), #mstatus
        (3860, 9), #mhartid
        (384, 10), #satp
        (944, 11), #pmpaddr0
        (928, 12), #pmpcfg0
        (770, 13), #medeleg
        (771, 14), #mideleg
        (1860, 15), #unknown
        ]
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
        
        is_memory = memory_read | memory_write
            
        # This `is_memory` hack is to evade rust's overflow check.
        addr = (result.bitcast(UInt(32)) - is_memory.select(data_offset, UInt(32)(0))).bitcast(Bits(32))
        request_addr = is_memory.select(addr[2:2+depth_log-1].bitcast(UInt(depth_log)), UInt(depth_log)(0))
        pc0 = (fetch_addr.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))

        with Condition( ~predict_wrong[0]):
            with Condition(produced_by_exec):
                scoreboard[sb_index] = modify_entry_exe(scoreboard,sb_index,result,Bits(2)(3))
                ex_update = sb_index
                ex_data = result
                
            with Condition(signals.is_branch):
                br_dest = condition[0:0].select(result, pc0)
                br_sm = condition[0:0].select(Bits(1)(1),Bits(1)(0))
                predict_wrong[0] =condition[0:0].select(Bits(1)(0),Bits(1)(1))
                execution_index = sb_index 
                log("condition: {}.a.b | a: {:08x}  | b: {:08x}   |", condition[0:0], result, pc0)
                
            
            with Condition(memory_read):
                log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)
            with Condition(memory_write):
                log("mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} | wdada: 0x{:08x}", result, request_addr, a, b)
            
            with Condition(is_memory):
                scoreboard[sb_index] =modify_entry_mem_status(scoreboard=scoreboard,index=sb_index,is_memory_read = memory_read,
                                result = signals.link_pc.select(pc0, result),
                                is_csr = signals.csr_write,
                                csr_id = csr_id,
                                csr_new = csr_new,
                                mem_ext = signals.mem_ext,
                                status=Bits(2)(2),
                                )
            
            dcache = SRAM(width=32, depth=1<<depth_log, init_file=data)
            dcache.name = 'dcache'
            dcache.build(we=memory_write, re=memory_read, wdata=b, addr=request_addr, user=memory)
            bound = dcache.bound.bind(rd=rd,index=sb_index,result = signals.link_pc.select(pc0, result) )
            
            bound.async_called()
        
            wb = writeback.bind()
            with Condition(rd != Bits(5)(0)):
                log("own x{:02}          |", rd)

        with Condition( predict_wrong[0]):
            predict_wrong[0] = Bits(1)(0)
        
        return  br_sm, br_dest, wb, rd, ex_valid,ex_update,execution_index,ex_data





class Decoder(Module):
    
    def __init__(self):
        super().__init__(ports={
            'rdata': Port(Bits(32)),
            'fetch_addr': Port(Bits(32)),
        })
        self.name = 'D'
        
    @module.combinational
    def build(self,   scoreboard:Array,RMT:Array,sb_head:Array,sb_tail:Array):
        
        inst = self.rdata.peek()
        fetch_addr = self.fetch_addr.peek()

        log("raw: 0x{:08x}  | addr: 0x{:05x} |", inst, fetch_addr)
        
        is_not_full_scoreboard =(((sb_tail[0].bitcast(UInt(32))+UInt(32)(1) ).bitcast(Bits(SCOREBOARD.Bit_size)) )& (Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size-1 ))) != sb_head[0] 
        decode_allowed = is_not_full_scoreboard.select(Bits(1)(1),Bits(1)(0))
        wait_until(decode_allowed)
        inst, fetch_addr = self.pop_all_ports(False)
            
        is_nop = ((inst == Bits(32)(51)) | (inst==Bits(32)(0))|(inst == Bits(32)(0xc0001073))).select(Bits(1)(1),Bits(1)(0))
        Index = sb_tail[0]
        
        with Condition(~is_nop):
            signals = decode_logic(inst)
            rmt_update_rd = (signals.rd_valid).select(signals.rd,Bits(5)(0))
            rmt_update_index = (signals.rd_valid).select(Index,Bits(SCOREBOARD.Bit_size)(0))
            decode_signals = signals.value()
            decode_index = Index
            decode_fetch_addr = fetch_addr
            is_br=signals.is_branch
            is_ebreak = (signals.rs1_valid & signals.imm_valid & ((signals.imm == Bits(32)(1))|(signals.imm == Bits(32)(0))) & (signals.alu == Bits(16)(0)))
            with Condition(is_br & (~is_ebreak)):
                predicted_fet_addr =( (signals.imm).bitcast(Int(32)) + fetch_addr.bitcast(Int(32)) ).bitcast(Bits(32))
        log("tail {:05} | is_not_full_scoreboard {:07} |  decode_allowed {} |",  Index,  is_not_full_scoreboard, decode_allowed) 
        
        return is_nop ,decode_allowed, is_br,rmt_update_rd,rmt_update_index,decode_index,decode_fetch_addr,decode_signals,predicted_fet_addr

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

class FetcherImpl(Downstream):

    def __init__(self):
        super().__init__()
        self.name = 'F1'

    @downstream.combinational
    def build(self,
              br_sm:Value,
              on_branch: Value, 
              br_signal:Array,
              ex_bypass: Value,
              ex_valid: Value,
              pc_reg: Value,
              pc_addr: Value,
              decoder: Decoder,
              data: str,
              depth_log: int,
              is_nop:Value,
              predicted_addr:Value):

        ongoing = RegArray(Int(8), 1, initializer=[0])

        br_sm_value = br_sm.optional(Bits(1)(0))
        aborted = (br_sm.valid())&(~br_sm_value)
        
        ex_bypass_value = ex_bypass.optional(Bits(32)(0))
        # on_branch = on_branch.optional(Bits(1)(0))   | br_signal[0]
        # should_fetch = ~on_branch | ex_bypass.valid()
        
        to_fetch = predicted_addr.optional(pc_addr)
        to_fetch = (aborted).select( ex_bypass_value, to_fetch)
        icache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        icache.name = 'icache'

        update_cnt = ex_valid.optional(Bits(1)(0)).bitcast(Int(8)) + is_nop.optional(Bits(1)(0)).bitcast(Int(8)) 
        new_cnt = ongoing[0] - update_cnt
         
        
        real_fetch = (new_cnt < Int(8)(15)).select(Bits(1)(1),Bits(1)(0))  
        log("  ex_by: {}     | fetch: {}      | addr: 0x{:05x} | ongoing: {}  ex_by_value {}   predicted_addr {}",
              ex_bypass.valid(), real_fetch, to_fetch, new_cnt,ex_bypass_value, predicted_addr.optional(pc_addr))

        icache.build(Bits(1)(0), real_fetch, to_fetch[2:2+depth_log-1].bitcast(Int(depth_log)), Bits(32)(0), decoder)
        
        with Condition(real_fetch):
            icache.bound.async_called(fetch_addr=to_fetch)
            pc_reg[0] = (to_fetch.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
            ongoing[0] =   new_cnt + Int(8)(1) # is_nop X
        
        with Condition(~real_fetch):
            pc_reg[0] = to_fetch
            ongoing[0] = new_cnt

class UpdateScoreboard(Downstream):

    def __init__(self):
        super().__init__()
        self.name = 'U'

    @downstream.combinational
    def build(self,
              cycle_activate:Value,
              mem:Value,
              ex:Value,
              scoreboard: Array,
              RMT: Array ,
              execution_index:Value ,
              is_nop:Value ,
              sb_tail:Array,
              br_sm:Value ,
              decode_allowed:Value,
              decode_on_branch:Value,
              rmt_update_rd:Value,
              rmt_update_index:Value,
              rmt_clear_rd:Value,
              rmt_clear_index:Value,
              ex_data:Value,
              mdata:Value,
              reg_file:Array,
              cur_index:Value,
              fetch_addr:Value,
              d_signals:Value):
              
        
        cycle_activate = cycle_activate.optional(Bits(1)(0)) 
        br_signal = RegArray(Bits(1), 1)
        
        is_branch = br_sm.optional(Bits(1)(0))  
        br_signal[0] = (br_sm.valid()).select( Bits(1)(0) , br_signal[0] | decode_on_branch.optional(Bits(1)(0)) )
        decoded_allowed = decode_allowed.optional(Bits(1)(0))
        not_decoded =  is_nop.optional(Bits(1)(0)) | (~decoded_allowed)
        update_tail = (not_decoded ).select(sb_tail[0],   ((((sb_tail[0].bitcast(UInt(32)) )+UInt(32)(1) ).bitcast(Bits(SCOREBOARD.Bit_size)) )& (Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size-1 ))))
        bypass_tail =  ((((execution_index.optional(sb_tail[0])).bitcast(UInt(32)) )+UInt(32)(1) ).bitcast(Bits(SCOREBOARD.Bit_size)) )& (Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size-1 ))
        
        sb_tail[0] = ( (~is_branch)&(br_sm.valid())).select( bypass_tail ,update_tail )
        
        rmt_clear_rd = rmt_clear_rd.optional(Bits(5)(0))
        rmt_up_rd = rmt_update_rd.optional(Bits(5)(0))
        mem_index = mem.optional(NoDep)
        ex_index = ex.optional(NoDep)  
        rmt_cl_index = rmt_clear_index.optional(NoDep)

        rmt_up_index=rmt_update_index.valid().select(rmt_update_index ,NoDep)
        RMT[rmt_up_rd] = rmt_up_index

        cur_index = cur_index.optional(NoDep)
        Fetch_addr = fetch_addr.optional(Bits(32)(0))
        signals = deocder_signals.view(d_signals.optional(Bits(97)(0)))
        e_data = ex_data.optional(Bits(32)(0))
        m_data = mdata.optional(Bits(32)(0))
        
        with Condition(~(is_nop.optional(Bits(1)(0))) & (cur_index!=NoDep)):
            newest_index = cur_index
            newest_entry = add_entry(signals,scoreboard,cur_index,RMT,reg_file,Fetch_addr,mem_index,ex_index,e_data,m_data)
            entry_value= newest_entry.value()
            
            is_ebreak = (signals.rs1_valid & signals.imm_valid & ((signals.imm == Bits(32)(1))|(signals.imm == Bits(32)(0)))\
                        & (signals.alu == Bits(16)(0))).select(Bits(1)(1),Bits(1)(0))
            
            early_dispatch_valid =( newest_entry.rs1_ready & newest_entry.rs2_ready &(~is_ebreak)).select(Bits(1)(1),Bits(1)(0))

        for i in range(SCOREBOARD.size):
            with Condition((scoreboard[i].sb_valid & (scoreboard[i].sb_status==Bits(2)(0))& \
                            (~( ((Bits(SCOREBOARD.Bit_size)(i)==bypass_tail))&(is_branch) ) )&(cur_index!=Bits(SCOREBOARD.Bit_size)(i)) )):
                
                rs1_dp = scoreboard[i].rs1_dep
                rs2_dp = scoreboard[i].rs2_dep
                with Condition((~scoreboard[i].rs1_ready)&(( rs1_dp == mem_index)|(rs1_dp == ex_index)) &(scoreboard[rs1_dp].sb_status!= Bits(2)(3))):
                    result = ( rs1_dp == mem_index).select(m_data,e_data)
                    scoreboard[i]=modify_entry_rs1(scoreboard,i,result)
                with Condition((~scoreboard[i].rs2_ready)&(( rs2_dp == mem_index)|(rs2_dp == ex_index)) &(scoreboard[rs2_dp].sb_status!= Bits(2)(3)) ):
                    result = ( rs2_dp == mem_index).select(m_data,e_data)
                    scoreboard[i]=modify_entry_rs2(scoreboard,i,result)
                
                rs1_update = (~scoreboard[i].rs1_ready)&(( scoreboard[rs1_dp].sb_status== Bits(2)(3)))
                rs2_update = (~scoreboard[i].rs2_ready)&(( scoreboard[rs2_dp].sb_status== Bits(2)(3)))
                with Condition(rs1_update|rs2_update):
                    scoreboard[i]=modify_entry_rs(scoreboard,i,rs1_update,rs2_update)
                
        with Condition( (rmt_clear_rd != Bits(5)(0))& (rmt_clear_rd!=rmt_up_rd)& (RMT[rmt_clear_rd]==rmt_cl_index)):
            RMT[rmt_clear_rd] = NoDep
            
            
        return  br_signal,newest_index,entry_value,early_dispatch_valid
 

class Dispatch(Downstream):

    def __init__(self):
        super().__init__()
        self.name = 'p'

    @downstream.combinational
    def build(self,scoreboard:Array,executor:Module,RMT:Array  ,trigger:Value,new_index:Value,new_entry_value:Value,early_dispatch_valid:Value): 
        
        trigger = trigger.optional(Bits(1)(0))
        early_dispatch_valid = early_dispatch_valid.optional(Bits(1)(0))
        new_entry_valid = new_index.valid()
        new_index = new_index.optional(NoDep)
        new_entry_value = new_entry_value.optional(Bits(318+2*SCOREBOARD.Bit_size)(0))
        with Condition(new_entry_valid):
            new_entry=scoreboard_entry.view(new_entry_value)
            update_status_entry = modify_entry_sb_status(new_entry)
            updated_entry_value = early_dispatch_valid.select( update_status_entry.value(), new_entry_value)
            wait_to_add_new_entry = scoreboard_entry.view(updated_entry_value)
            scoreboard[new_index] = wait_to_add_new_entry

            with Condition(early_dispatch_valid):
                for i in range(SCOREBOARD.size):
                    log("i {}, addr {} valid {}  status {}  rs1 {} rs2 {} dep1 {} dep2 {} |",\
                    Bits(6)(i), scoreboard[i].fetch_addr ,scoreboard[i].sb_valid ,scoreboard[i].sb_status ,\
                    scoreboard[i].rs1_ready,scoreboard[i].rs2_ready,RMT[scoreboard[i].rs1], RMT[scoreboard[i].rs2])
                
                log("Dispatch call execution index {:05} | ",  new_index )
                call = executor.async_called(rs1_value=new_entry.rs1_value,rs2_value=new_entry.rs2_value ,\
                                                        signals= new_entry.signals,fetch_addr=new_entry.fetch_addr ,sb_index=new_index)
                
                call.bind.set_fifo_depth()

        with Condition(~early_dispatch_valid):
            valid_global = Bits(1)(0)  # check if there is a valid entry to be executed
            valid_temp = Bits(1)(0)
            not_ready = Bits(1)(0)
            ebreak_index = Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
            second_dispatch_index = Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
            dispatch_index = Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
            for i in range(SCOREBOARD.size):
                valid_temp =  (  scoreboard[i].sb_valid & (scoreboard[i].sb_status==Bits(2)(0)) & scoreboard[i].rs1_ready & scoreboard[i].rs2_ready   )  
                second_dispatch_index = valid_temp.select(dispatch_index, second_dispatch_index)
                dispatch_index = valid_temp.select(Bits(SCOREBOARD.Bit_size)(i), dispatch_index)
                log("i {}, addr {} valid {}  status {}   dispatch {}   rs1 {} rs2 {} dep1 {} dep2 {} |",\
                    Bits(6)(i), scoreboard[i].fetch_addr ,scoreboard[i].sb_valid ,scoreboard[i].sb_status ,dispatch_index,\
                    scoreboard[i].rs1_ready,scoreboard[i].rs2_ready,RMT[scoreboard[i].rs1], RMT[scoreboard[i].rs2])
                signals= deocder_signals.view(scoreboard[i].signals)
                is_ebreak_temp = (signals.rs1_valid & signals.imm_valid & ((signals.imm == Bits(32)(1))|(signals.imm == Bits(32)(0))) & (signals.alu == Bits(16)(0)))
                ebreak_index = is_ebreak_temp.select(  Bits(SCOREBOARD.Bit_size)(i) , ebreak_index)
                not_ready = not_ready | ((scoreboard[i].sb_valid )& (~is_ebreak_temp))
            
            signals= deocder_signals.view(scoreboard[dispatch_index].signals)
            is_ebreak = (signals.rs1_valid & signals.imm_valid & ((signals.imm == Bits(32)(1))|(signals.imm == Bits(32)(0)))\
                        & (signals.alu == Bits(16)(0))).select(Bits(1)(1),Bits(1)(0))
            dispatch_index = (is_ebreak).select(second_dispatch_index,dispatch_index)
            
            valid_global =  (dispatch_index!= NoDep) 
            with Condition(is_ebreak & (~not_ready) ):
                log('ebreak | halt | ecall')
                finish()
            
            with Condition(valid_global ):
                scoreboard[dispatch_index] =modify_entry_status(scoreboard,dispatch_index,Bits(2)(1))
                log("Dispatch call execution index {:05} | ",  dispatch_index )
                signals=deocder_signals.view(scoreboard[dispatch_index].signals)
                
                call = executor.async_called(rs1_value=scoreboard[dispatch_index].rs1_value,rs2_value=scoreboard[dispatch_index].rs2_value ,\
                                                        signals= scoreboard[dispatch_index].signals,fetch_addr=scoreboard[dispatch_index].fetch_addr ,sb_index=dispatch_index)
                
                call.bind.set_fifo_depth()
            
class Driver(Module):
    
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, fetcher: Module):
        fetcher.async_called()


def run_cpu(resource_base, workload, depth_log):
    sys = SysBuilder('o3_cpu')

    with sys:

        with open(f'{resource_base}/{workload}.config') as f:
            global offset, data_offset
            raw = f.readline()
            raw = raw.replace('offset:', "'offset':").replace('data_offset:', "'data_offset':")
            offsets = eval(raw)
            print(offsets)
            offset = offsets['offset']
            data_offset = offsets['data_offset']
            offset = UInt(32)(offset)
            data_offset = UInt(32)(data_offset)

        # Data Types
        bits1   = Bits(1)
        bits5   = Bits(5)
        bits32  = Bits(32)

        fetcher = Fetcher()
        pc_reg, pc_addr ,cycle_activate= fetcher.build()

        fetcher_impl = FetcherImpl()

        # Data Structures
        reg_file    = RegArray(bits32, 32)

        reg_map_table = RegArray(Bits(SCOREBOARD.Bit_size),33,initializer=[SCOREBOARD.size]*33,attr=[Array.FULLY_PARTITIONED])

    
        scoreboard = RegArray(scoreboard_entry,SCOREBOARD.size,attr=[Array.FULLY_PARTITIONED])

        sb_head = RegArray(Bits(SCOREBOARD.Bit_size), 1, initializer=[0])
        sb_tail = RegArray(Bits(SCOREBOARD.Bit_size), 1, initializer=[0])


        csr_file = RegArray(Bits(32), 16, initializer=[0]*16)


        writeback = WriteBack()
        wb_rd ,rmt_clear_rd,rmt_clear_index= writeback.build(reg_file = reg_file , csr_file = csr_file,scoreboard=scoreboard,RMT=reg_map_table,sb_head=sb_head)

        memory_access = MemoryAccess()

        executor = Execution()
        
        data_init = f'{workload}.data' if os.path.exists(f'{resource_base}/{workload}.data') else None

        br_sm, ex_bypass, wb, exec_rd, ex_valid,ex_update,execution_index,ex_data = executor.build(
            pc = pc_reg,
            rf = reg_file,
            csr_f = csr_file,
            memory = memory_access,
            writeback = writeback,
            data = data_init,
            depth_log = depth_log,
            scoreboard=scoreboard,
            RMT=reg_map_table
            )
        
        
        mem_update,m_data = memory_access.build(
            writeback = wb, 
            scoreboard=scoreboard,
            RMT=reg_map_table
        )
        
        
        decoder = Decoder()
        
        update_sb = UpdateScoreboard()

        
        is_nop,decode_allowed ,decode_on_branch ,rmt_update_rd,rmt_update_index,decode_index,decode_fetch_addr,decode_signals,predicted_addr= decoder.build( \
             scoreboard=scoreboard, RMT=reg_map_table, sb_head=sb_head,sb_tail=sb_tail)


        br_signal,newest_index,entry_value,early_dispatch_valid= update_sb.build(cycle_activate=cycle_activate ,ex=ex_update,mem=mem_update,scoreboard=scoreboard,RMT=reg_map_table,execution_index=execution_index ,is_nop=is_nop,\
            sb_tail=sb_tail,br_sm=br_sm,decode_allowed=decode_allowed,decode_on_branch=decode_on_branch ,rmt_clear_rd=rmt_clear_rd,rmt_clear_index=rmt_clear_index,\
                rmt_update_rd=rmt_update_rd,rmt_update_index=rmt_update_index,mdata=m_data,ex_data = ex_data,reg_file=reg_file,\
                      cur_index=decode_index, fetch_addr=decode_fetch_addr,d_signals=decode_signals)
        
        fetcher_impl.build(br_sm,decode_on_branch,br_signal, ex_bypass, ex_valid, pc_reg, pc_addr, decoder, f'{workload}.exe', depth_log,is_nop,predicted_addr)
        
        dispatch = Dispatch()
        dispatch.build(executor=executor,scoreboard=scoreboard,RMT=reg_map_table,trigger=cycle_activate,new_index=newest_index,new_entry_value=entry_value,early_dispatch_valid=early_dispatch_valid) 
        
        driver = Driver()
        driver.build(fetcher)

        
        

    print(sys)
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=1000000,
        idle_threshold=1000000,
        resource_base=resource_base
    )

    # simulator_path = elaborate(sys, **conf)
    simulator_path, verilog_path = elaborate(sys, **conf)

    report = False

    if report:
        raw, tt = utils.run_simulator(simulator_path, True)
        open(f'{workload}.log', 'w').write(raw)
        open(f'{workload}.sim.time', 'w').write(str(tt))
        # raw, tt = utils.run_verilator(verilog_path, True)
        # open(f'{workload}.verilog.log', 'w').write(raw)
    else:
        raw = utils.run_simulator(simulator_path, False)
        open('raw.log', 'w').write(raw)
        check(resource_base, workload)
        # raw = utils.run_verilator(verilog_path, False)
        # open('raw.log', 'w').write(raw)
        check(resource_base, workload)
        os.remove('raw.log')



def check(resource_base, test):

    script = f'{resource_base}/{test}.sh'
    if os.path.exists(script):
        res = subprocess.run([script, 'raw.log', f'{resource_base}/{test}.data'])
    else:
        script = f'{resource_base}/../utils/find_pass.sh'
        res = subprocess.run([script, 'raw.log'])
    assert res.returncode == 0, f'Failed test {test}'
    print('Test passed!!!')
    

if __name__ == '__main__':
    wl_path = f'{utils.repo_path()}/examples/o3-cpu/workloads'
    workloads = [
        '0to100',
        #'dhrystone',
        #'median',
        #'multiply',
        #'qsort',
        #'rsort',
        #'towers',
        #'vvadd',
    ]
    for wl in workloads:
        run_cpu(wl_path, wl, 16)

    test_cases = [
        # 'rv32ui-p-add',
        # 'rv32ui-p-addi',
        # 'rv32ui-p-and',
        # 'rv32ui-p-andi',
        # 'rv32ui-p-auipc',
        # 'rv32ui-p-beq',
        # 'rv32ui-p-bge',
        # 'rv32ui-p-bgeu',
        # 'rv32ui-p-blt',
        # 'rv32ui-p-bltu',
        # 'rv32ui-p-bne',
        # 'rv32ui-p-jal',
        # 'rv32ui-p-jalr',
        # 'rv32ui-p-lui',
        # 'rv32ui-p-lw',
        # 'rv32ui-p-or',
        # 'rv32ui-p-ori',
        # 'rv32ui-p-sll',
        # 'rv32ui-p-slli',
        # 'rv32ui-p-sltu',
        # 'rv32ui-p-srai',
        # 'rv32ui-p-srl',
        # 'rv32ui-p-srli',
        # 'rv32ui-p-sub',
        # 'rv32ui-p-sw',
        # 'rv32ui-p-xori',

        #'rv32ui-p-lbu',#TO DEBUG&TO CHECK
        #'rv32ui-p-sb',#TO CHECK
    ]

    tests = f'{utils.repo_path()}/examples/minor-cpu/unit-tests'

    for case in test_cases:
        run_cpu(tests, case, 9)

