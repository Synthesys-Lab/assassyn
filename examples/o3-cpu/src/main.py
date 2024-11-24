''' A simplest single issue RISCV CPU, which has no operand buffer.
'''

import pytest
import os
import sys


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

        log("In execution")

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

        ex_valid = (signals.rs1_valid & signals.rs2_valid  )



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

        is_ebreak = signals.rs1_valid & signals.imm_valid & ((signals.imm == Bits(32)(1))|(signals.imm == Bits(32)(0))) & (signals.alu == Bits(16)(0))
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
        
        # Broadcast scoreboard
        with Condition(produced_by_exec):
            for id in range(SCOREBOARD.size):
                Idx=UInt(5)(id)
                with Condition(scoreboard[id].sb_valid & ~scoreboard[id].sb_status):
                    log(" rs1_ready {:03x}  rs2_ready {:03x}  rs1_dep {:06} rs2_dep {:06} rmt[rd] {:06} ",\
                         scoreboard[id].rs1_ready,scoreboard[id].rs2_ready ,  scoreboard[id].rs1_dep ,\
                             scoreboard[id].rs2_dep ,RMT[rd]  )
                    rs1_update = ( ~scoreboard[id].rs1_ready & (scoreboard[id].rs1_dep== RMT[rd])  ).select(Bits(1)(1),Bits(1)(0))
                    rs2_update= ( ~scoreboard[id].rs2_ready & (scoreboard[id].rs2_dep== RMT[rd] ) ).select(Bits(1)(1),Bits(1)(0)) 
                    with Condition (rs1_update|rs2_update):
                        scoreboard[id] = modify_entry_rs(scoreboard,id,rs1_update,rs2_update,result)   
                    
        with Condition(signals.rd_valid):
            log("write RMT rd {:02}      index {:07}     |", rd,sb_index)
            RMT[rd]=Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size)
            log("finish write RMT")
        
        pc0 = (fetch_addr.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
        with Condition(signals.is_branch):
            br_dest = condition[0:0].select(result, pc0)
            log("condition: {}.a.b | a: {:08x}  | b: {:08x}   |", condition[0:0], result, pc0)
            br_sm = RegArray(Bits(1), 1)
            br_sm[0] = Bits(1)(0)

        is_memory = memory_read | memory_write

        # This `is_memory` hack is to evade rust's overflow check.
        addr = (result.bitcast(UInt(32)) - is_memory.select(data_offset, UInt(32)(0))).bitcast(Bits(32))
        request_addr = is_memory.select(addr[2:2+depth_log-1].bitcast(UInt(depth_log)), UInt(depth_log)(0))

        with Condition(memory_read):
            log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)

        with Condition(memory_write):
            log("mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} | wdada: 0x{:08x}", result, request_addr, a, b)

        dcache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        dcache.name = 'dcache'
        dcache.build(we=memory_write, re=memory_read, wdata=b, addr=request_addr, user=memory)
        with Condition(memory_read):
            bound = dcache.bound.bind(rd=rd)
        bound.async_called()
        wb = writeback.bind(is_memory_read = memory_read,
                            result = signals.link_pc.select(pc0, result),
                            rd = rd,
                            is_csr = signals.csr_write,
                            csr_id = csr_id,
                            csr_new = csr_new,
                            mem_ext = signals.mem_ext,
                            index=sb_index)

        with Condition(rd != Bits(5)(0)):
            log("own x{:02}          |", rd)

        log("out execution")
        return br_sm, br_dest, wb, rd, ex_valid



class Dispatch(Module):

    def __init__(self):
        super().__init__(
            ports={}
        )
        self.name = 'p'

    @module.combinational
    def build(self,scoreboard:Array,executor:Module,RMT:Array  ):
        log("In dispatch")
        
        dispatch_index = Bits(SCOREBOARD.Bit_size)(1)
        valid_global = Bits(1)(0)  # check if there is a valid entry to be executed
        valid_temp =Bits(1)(0)
        for i in range(SCOREBOARD.size):
            valid_temp =  (  scoreboard[i].sb_valid & ~scoreboard[i].sb_status & scoreboard[i].rs1_ready & scoreboard[i].rs2_ready  )  
            valid_global = valid_global | valid_temp 
            dispatch_index = valid_temp.select(Bits(SCOREBOARD.Bit_size)(i), dispatch_index)
            # log("i {:06}, sb_status {:04} valid_temp  {:07}  dispatch index {:07}  scoreboard[i].rs1_ready {:07} |",Bits(6)(i),scoreboard[i].sb_status, valid_temp ,dispatch_index,\
                # scoreboard[i].rs1_ready)
        
        with Condition(Bits(1)(1)):
             
            scoreboard[dispatch_index] =modify_entry_status(scoreboard,dispatch_index,Bits(1)(1))

            log("Dispatch call execution index {:05}  sb_status {:07}| ",  dispatch_index, scoreboard[dispatch_index].sb_status)
            call = executor.async_called(rs1_value=scoreboard[dispatch_index].rs1_value,rs2_value=scoreboard[dispatch_index].rs2_value ,\
                                                    signals= scoreboard[dispatch_index].signals,fetch_addr=scoreboard[dispatch_index].fetch_addr ,sb_index=dispatch_index)
             
            call.bind.set_fifo_depth()
        log("out Dispatch")



class Decoder(Module):
    
    def __init__(self):
        super().__init__(ports={
            'rdata': Port(Bits(32)),
            'fetch_addr': Port(Bits(32)),
        })
        self.name = 'D'

    @module.combinational
    def build(self, Dispatch: Module, br_sm: Array, scoreboard:Array,RMT:Array,reg_file:Array,sb_head:Array,sb_tail:Array ):
        inst, fetch_addr = self.pop_all_ports(False)
        log("In decoder")
        log("raw: 0x{:08x}  | addr: 0x{:05x} |", inst, fetch_addr)

        signals = decode_logic(inst)
        br_sm[0] = signals.is_branch
        
        is_not_full_scoreboard =( ((sb_tail[0].bitcast(UInt(32))+UInt(32)(1) )%SCOREBOARD.sizeI ) != sb_head[0].bitcast(UInt(32)) )
        wait_until( is_not_full_scoreboard )
        Index = sb_tail[0]
        with Condition(signals.rd_valid):
            noWAW =  ( RMT[signals.rd] ==Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size) ).select(Bits(1)(1),Bits(1)(0))
            wait_until(noWAW)
        RMT[signals.rd]= (signals.rd_valid).select(Index,Bits(SCOREBOARD.Bit_size)(SCOREBOARD.size) )

        scoreboard[Index] = add_entry(signals,scoreboard,Index,RMT,reg_file,fetch_addr)
        sb_tail[0]= (((sb_tail[0].bitcast(UInt(32)) )+UInt(32)(1) )%SCOREBOARD.sizeI ).bitcast(Bits(SCOREBOARD.Bit_size))
        log("update sb_tail {:05}",sb_tail[0])
        Dispatch.async_called()

        # log("out decoder")
        return signals.is_branch

class Fetcher(Module):
    
    def __init__(self):
        super().__init__(ports={})
        self.name = 'F'

    @module.combinational
    def build(self):
        log("In Fetch")
        pc_reg = RegArray(Bits(32), 1)
        addr = pc_reg[0]
        
        return pc_reg, addr

class FetcherImpl(Downstream):

    def __init__(self):
        super().__init__()
        self.name = 'F1'

    @downstream.combinational
    def build(self,
              on_branch: Value,
              br_sm: Array,
              ex_bypass: Value,
              ex_valid: Value,
              pc_reg: Value,
              pc_addr: Value,
              decoder: Decoder,
              data: str,
              depth_log: int):
        log("In fetchImpl")
        ongoing = RegArray(Int(8), 1, initializer=[0])
        
        on_branch = on_branch.optional(Bits(1)(0)) | br_sm[0]
        should_fetch = ~on_branch | ex_bypass.valid()
        to_fetch = ex_bypass.optional(pc_addr)
        icache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        icache.name = 'icache'

        new_cnt = ongoing[0] - (ex_valid.optional(Bits(1)(0))).select(Int(8)(1), Int(8)(0))
        real_fetch = should_fetch & (new_cnt < Int(8)(5))

        icache.build(Bits(1)(0), real_fetch, to_fetch[2:2+depth_log-1].bitcast(Int(depth_log)), Bits(32)(0), decoder)
        log("on_br: {}         | ex_by: {}     | fetch: {}      | addr: 0x{:05x} | ongoing: {}",
            on_branch, ex_bypass.valid(), real_fetch, to_fetch, new_cnt)

        with Condition(real_fetch):
            icache.bound.async_called(fetch_addr=to_fetch)
            pc_reg[0] = (to_fetch.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
            ongoing[0] = new_cnt + Int(8)(1)
        
        with Condition(~real_fetch):
            pc_reg[0] = to_fetch
            ongoing[0] = new_cnt
        log("out fetchImpl")


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
        pc_reg, pc_addr = fetcher.build()

        fetcher_impl = FetcherImpl()

        # Data Structures
        reg_file    = RegArray(bits32, 32)



        reg_map_table = RegArray(Bits(SCOREBOARD.Bit_size),33,initializer=[SCOREBOARD.size]*33,attr=[Array.FULLY_PARTITIONED])


        scoreboard = RegArray(scoreboard_entry,SCOREBOARD.size,attr=[Array.FULLY_PARTITIONED])

        sb_head = RegArray(Bits(SCOREBOARD.Bit_size), 1, initializer=[0])
        sb_tail = RegArray(Bits(SCOREBOARD.Bit_size), 1, initializer=[0])


        csr_file = RegArray(Bits(32), 16, initializer=[0]*16)

        # exec_bypass_reg = RegArray(bits5, 1)
        # exec_bypass_data = RegArray(bits32, 1)

        # mem_bypass_reg = RegArray(bits5, 1)
        # mem_bypass_data = RegArray(bits32, 1)

        writeback = WriteBack()
        wb_rd = writeback.build(reg_file = reg_file , csr_file = csr_file,scoreboard=scoreboard,RMT=reg_map_table,sb_head=sb_head)

        memory_access = MemoryAccess()

        executor = Execution()
        
        data_init = f'{workload}.data' if os.path.exists(f'{resource_base}/{workload}.data') else None

        br_sm, ex_bypass, wb, exec_rd, ex_valid = executor.build(
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
        
        
        memory_access.build(
            writeback = wb, 
            scoreboard=scoreboard,
            RMT=reg_map_table
        )
        dispatch = Dispatch()
        dispatch.build(executor=executor,scoreboard=scoreboard,RMT=reg_map_table)
        
        decoder = Decoder()
        on_br = decoder.build(Dispatch=dispatch, br_sm=br_sm,scoreboard=scoreboard, RMT=reg_map_table,reg_file=reg_file,\
                              sb_head=sb_head,sb_tail=sb_tail )
        fetcher_impl.build(on_br, br_sm, ex_bypass, ex_valid, pc_reg, pc_addr, decoder, f'{workload}.exe', depth_log)

    
        driver = Driver()
        driver.build(fetcher)

    print(sys)
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=1000000,
        idle_threshold=1000000,
        resource_base=resource_base
    )

    simulator_path, verilog_path = elaborate(sys, **conf)

    report = False

    if report:
        raw, tt = utils.run_simulator(simulator_path, True)
        open(f'{workload}.log', 'w').write(raw)
        open(f'{workload}.sim.time', 'w').write(str(tt))
        raw, tt = utils.run_verilator(verilog_path, True)
        open(f'{workload}.verilog.log', 'w').write(raw)
    else:
        raw = utils.run_simulator(simulator_path, False)
        open('raw.log', 'w').write(raw)
        check(resource_base, workload)
        raw = utils.run_verilator(verilog_path, False)
        open('raw.log', 'w').write(raw)
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
    wl_path = f'{utils.repo_path()}/examples/minor-cpu/workloads'
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

    for case in test_cases:
        run_cpu(tests, case, 9)

