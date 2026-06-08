''' A simplest single issue RISCV CPU, which has no operand buffer.
'''
import argparse
import csv
import os
import re
import shutil
import subprocess

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

from opcodes import *
from decoder import *
from writeback import *
from memory_access import *
import sys as py_sys

offset = UInt(32)(0)
current_path = os.path.dirname(os.path.abspath(__file__))
workspace = f'{current_path}/.workspace/'

WORKLOAD_ALIASES = {
    'coremask': 'coremark',
}

class Execution(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'raw_inst': Port(Bits(32)),
                'signals': Port(deocder_signals),
                'fetch_addr': Port(Bits(32)),
                'epoch': Port(Bits(8)),
            })
        self.name = "E"
        #self.exe_valid = Bits(1)

    @module.combinational
    def build(
        self, 
        pc: Array, 
        exec_bypass_reg: Array,
        exec_bypass_data: Array,
        mem_bypass_reg: Array,
        mem_bypass_data: Array,
        wb_bypass_reg: Array,
        wb_bypass_data: Array,
        reg_onwrite: Array,
        offset_reg: Array,
        fetch_epoch: Array,
        rf: Array,
        csr_f: Array,
        memory: Module,
        data: str,
        depth_log: int,
        exec_br_dest: Array,
        exec_br_valid: Array,
        exec_br_taken: Array,
        ):

        csr_id = Bits(4)(0)
 
        signals = self.signals.peek()
        fetch_addr_peek = self.fetch_addr.peek()
        inst_epoch = self.epoch.peek()
        pending_redirect = exec_br_valid[0] & exec_br_taken[0]
        next_live_epoch = (fetch_epoch[0].bitcast(UInt(8)) + UInt(8)(1)).bitcast(Bits(8))
        live_epoch = pending_redirect.select(next_live_epoch, fetch_epoch[0])
        is_redirect_target = pending_redirect & (fetch_addr_peek == exec_br_dest[0])
        is_live = (inst_epoch == live_epoch) | is_redirect_target

        with Condition(pending_redirect | ~is_live):
            log(
                "epoch.chk       | pc: 0x{:08x} | inst: 0x{:02x} | fetch: 0x{:02x} | live: 0x{:02x} | pend: {} | taken: {} | dest: 0x{:08x} | tgt: {}",
                fetch_addr_peek,
                inst_epoch,
                fetch_epoch[0],
                live_epoch,
                pending_redirect,
                exec_br_taken[0],
                exec_br_dest[0],
                is_redirect_target,
            )

        rs1 = signals.rs1
        rs2 = signals.rs2
        rd = signals.rd

        on_write = reg_onwrite[0]

         

        a_valid =(exec_bypass_reg[0] == rs1) | (mem_bypass_reg[0] == rs1) | ~signals.rs1_valid | (~(on_write >> rs1))[0:0] #| (wb_bypass_reg[0] == rs1)
        a_valid_true = a_valid.select(Bits(1)(1),(wb_bypass_reg[0] == rs1).bitcast(Bits(1))) 
        with Condition(~a_valid):
            log("exec_bypass_reg: x{:02} | mem_bypass_reg: x{:02} | ~signals.rs1_valid: {} | (~(on_write >> rs1))[0:0]: {} |", exec_bypass_reg[0], mem_bypass_reg[0], ~signals.rs1_valid, (~(on_write >> rs1))[0:0])

        b_valid =(exec_bypass_reg[0] == rs2) | (mem_bypass_reg[0] == rs2) | ~signals.rs2_valid | (~(on_write >> rs2))[0:0] #| (wb_bypass_reg[0] == rs2)
        b_valid_true = b_valid.select(Bits(1)(1),(wb_bypass_reg[0] == rs2).bitcast(Bits(1) ))
        with Condition(~b_valid):
            log("exec_bypass_reg: x{:02} | mem_bypass_reg: x{:02} | ~signals.rs2_valid: {} | (~(on_write >> rs2))[0:0]: {} |", exec_bypass_reg[0], mem_bypass_reg[0], ~signals.rs2_valid, (~(on_write >> rs2))[0:0])

        rd_valid =  (exec_bypass_reg[0] == rd) | (mem_bypass_reg[0] == rd) | ~signals.rd_valid | (~(on_write >> rd))[0:0] 
        rd_valid_true = rd_valid.select(Bits(1)(1),(wb_bypass_reg[0] == rd).bitcast(Bits(1) ))
        with Condition(~rd_valid):
            log("exec_bypass_reg: x{:02} | mem_bypass_reg: x{:02} | ~signals.rd_valid: {} | (~(on_write >> rd))[0:0]: {} |", exec_bypass_reg[0], mem_bypass_reg[0], ~signals.rd_valid, (~(on_write >> rd))[0:0])

        valid = a_valid_true & b_valid_true & rd_valid_true
        valid = is_live.select(valid, Bits(1)(1))

        with Condition(~valid):
            log("pc: 0x{:08x}   | rs1-x{:02}: {}       | rs2-x{:02}: {}   | rd-x{:02}: {} | backlogged", \
                fetch_addr_peek, rs1, a_valid, rs2, b_valid, rd, rd_valid)
        with Condition(~is_live):
            log("drop stale      | pc: 0x{:08x} | inst_epoch: 0x{:02x} | cur_epoch: 0x{:02x}", \
                fetch_addr_peek, inst_epoch, live_epoch)

        valid = valid 
        wait_until(valid)
        ex_valid = valid
        self.exe_valid = ex_valid


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


        raw_inst, signals, fetch_addr, inst_epoch = self.pop_all_ports(False)
        

        # TODO(@were): This is a hack to avoid post wait_until checks.
        rd = signals.rd

        is_ebreak = signals.rs1_valid & signals.imm_valid & \
                    ((signals.imm == Bits(32)(1)) | (signals.imm == Bits(32)(0))) & \
                    (signals.alu == Bits(16)(1<<RV32I_ALU.ALU_NONE))        


        is_ebreak = is_live & is_ebreak
        with Condition(is_ebreak):
            log('ebreak | halt | ecall')
            finish()

        # Treat zero-offset self-loops (including `jal x0, 0`) as a terminal trap.
        # Several packaged workloads end in this sentinel loop instead of ebreak.
        is_trap = signals.is_branch & \
                  signals.is_offset_br & \
                  signals.imm_valid & \
                  (signals.imm == Bits(32)(0)) & \
                  (signals.cond == Bits(RV32I_ALU.CNT)(1 << RV32I_ALU.ALU_TRUE)) & \
                  (signals.alu == Bits(RV32I_ALU.CNT)(1 << RV32I_ALU.ALU_ADD))
        is_trap = is_live & is_trap
        with Condition(is_trap):
            log('trap')
            finish()

        # Instruction attributes

        def bypass(bypass_reg, bypass_data, idx, value):
            return (bypass_reg[0] == idx).select(bypass_data[0], value)

        a = bypass(wb_bypass_reg, wb_bypass_data, rs1, rf[rs1])
        a = bypass(mem_bypass_reg, mem_bypass_data, rs1, a)
        a = bypass(exec_bypass_reg, exec_bypass_data, rs1, a)
        a = (rs1 == Bits(5)(0)).select(Bits(32)(0), a)
        a = signals.csr_write.select(Bits(32)(0), a)

        b = bypass(wb_bypass_reg, wb_bypass_data, rs2, rf[rs2])
        b = bypass(mem_bypass_reg, mem_bypass_data, rs2, b)
        b = bypass(exec_bypass_reg, exec_bypass_data, rs2, b)
        b = (rs2 == Bits(5)(0)).select(Bits(32)(0), b)
        b = is_csr.select(csr_f[csr_id], b)
        

        with Condition(is_live):
            log('decoded         | raw: 0x{:08x}', raw_inst)
            log('mem_bypass.reg: x{:02} | .data: {:08x}', mem_bypass_reg[0], mem_bypass_data[0])
            log('exe_bypass.reg: x{:02} | .data: {:08x}', exec_bypass_reg[0], exec_bypass_data[0])

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
        results[RV32I_ALU.ALU_NONE] = Bits(32)(0)

        # TODO: Fix this bullshit.
        alu = signals.alu
        result = alu.select1hot(*results)

        with Condition(is_live):
            log('pc: 0x{:08x}   |is_offset_br: {}| is_pc_calc: {}|', fetch_addr, signals.is_offset_br, signals.is_pc_calc)
            log("0x{:08x}       | a: {:08x}  | b: {:08x}   | imm: {:08x} | result: {:08x}", alu, a, b, signals.imm, result)
            log("0x{:08x}       |a.a:{:08x}  |a.b:{:08x}   | res: {:08x} |", alu, alu_a, alu_b, result)

        condition = signals.cond.select1hot(*results)
        condition = signals.flip.select(~condition, condition)

        memory_read = is_live & signals.memory[0:0]
        memory_write = is_live & signals.memory[1:1]
        live_rd = is_live.select(rd, Bits(5)(0))

        # TODO: Make this stricter later.
        produced_by_exec = is_live & ~memory_read & (rd != Bits(5)(0))
        exec_bypass_reg[0] = produced_by_exec.select(rd, Bits(5)(0))
        exec_bypass_data[0] = produced_by_exec.select(result, Bits(32)(0))

  
        pc0 = (fetch_addr.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
        branch_resolved = is_live & signals.is_branch
        branch_target = condition[0:0].select(result, pc0)
        with Condition(branch_resolved):
            exec_br_dest[0] = branch_target
            log("condition: {}.a.b | a: {:08x}  | b: {:08x}   |", condition[0:0], result, pc0)
        with Condition(is_live):
            exec_br_valid[0] = signals.is_branch
            exec_br_taken[0] = signals.is_branch.select(condition[0:0], Bits(1)(0))

        is_memory = memory_read | memory_write

        # This `is_memory` hack is to evade rust's overflow check.
        addr = (result.bitcast(UInt(32)) - is_memory.select(offset_reg[0].bitcast(UInt(32)), UInt(32)(0))).bitcast(Bits(32))
        addr_lsb = addr[0:1]
        request_addr = is_memory.select(addr[2:2+depth_log-1].bitcast(UInt(depth_log)), UInt(depth_log)(0))

        with Condition(memory_read):
            log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)

        with Condition(memory_write):
            log("mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} | wdada: 0x{:08x}", result, request_addr, a, b)

        byte_wmask = Bits(32)(0x000000ff)
        byte_wdata = Bits(24)(0).concat(b[0:7])
        byte_wmask = (addr_lsb == Bits(2)(1)).select(Bits(32)(0x0000ff00), byte_wmask)
        byte_wdata = (addr_lsb == Bits(2)(1)).select(Bits(16)(0).concat(b[0:7]).concat(Bits(8)(0)), byte_wdata)
        byte_wmask = (addr_lsb == Bits(2)(2)).select(Bits(32)(0x00ff0000), byte_wmask)
        byte_wdata = (addr_lsb == Bits(2)(2)).select(Bits(8)(0).concat(b[0:7]).concat(Bits(16)(0)), byte_wdata)
        byte_wmask = (addr_lsb == Bits(2)(3)).select(Bits(32)(0xff000000), byte_wmask)
        byte_wdata = (addr_lsb == Bits(2)(3)).select(b[0:7].concat(Bits(24)(0)), byte_wdata)

        half_wmask = addr_lsb[1:1].select(Bits(32)(0xffff0000), Bits(32)(0x0000ffff))
        half_wdata = addr_lsb[1:1].select(b[0:15].concat(Bits(16)(0)), Bits(16)(0).concat(b[0:15]))

        is_half = signals.mem_size == Bits(2)(1)
        is_byte = signals.mem_size == Bits(2)(2)
        store_wmask = is_byte.select(byte_wmask, is_half.select(half_wmask, Bits(32)(0xffffffff)))
        store_wdata = is_byte.select(byte_wdata, is_half.select(half_wdata, b))

        dcache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        dcache.name = 'dcache'
        dcache.build(we=memory_write, re=memory_read, wdata=store_wdata, addr=request_addr, wmask=store_wmask)
        with Condition(is_live):
            m_call = memory.async_called(
                rd=live_rd,
                result=signals.link_pc.select(pc0, result),
                mem_size=signals.mem_size,
                mem_unsigned=signals.mem_unsigned,
                addr_lsb=addr_lsb,
                is_mem_read=memory_read,
            )
            m_call.bind.set_fifo_depth(
                rd=2,
                result=2,
                mem_size=2,
                mem_unsigned=2,
                addr_lsb=2,
                is_mem_read=2,
            )
        with Condition(is_live & signals.csr_write):
            csr_f[csr_id] = csr_new

        with Condition(is_live & (rd != Bits(5)(0))):
            log("own x{:02}          |", rd)

        return live_rd, ex_valid, dcache

class Decoder(Module):
    
    def __init__(self):
        super().__init__(ports={
            'raw_inst': Port(Bits(32)),
            'fetch_addr': Port(Bits(32)),
            'epoch': Port(Bits(8)),
        })
        self.name = 'D'

    @module.combinational
    def build(self, executor: Module):
        raw_inst, fetch_addr, epoch = self.pop_all_ports(False)

        log("raw: 0x{:08x}  | addr: 0x{:05x} |", raw_inst, fetch_addr)

        signals = decode_logic(raw_inst)
 
        
        e_call = executor.async_called(raw_inst=raw_inst, signals=signals, fetch_addr=fetch_addr, epoch=epoch)
        e_call.bind.set_fifo_depth(raw_inst=2, signals=2, fetch_addr=2, epoch=2)

        return signals.is_branch



class Fetcher(Module):
    
    def __init__(self):
        super().__init__(ports={}, no_arbiter=True)
        self.name = 'F'

    @module.combinational
    def build(self):
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
              ex_bypass: Array,
              ex_valid: Value,
              pc_reg: Value,
              pc_addr: Value,
              decoder: Decoder,
              depth_log: int,
              br_sm: Array,
              exec_br_valid: Array,
              exec_br_taken: Array,
              fetch_epoch: Array,
              icache: SRAM,

              ):

        ongoing = RegArray(Int(8), 1, initializer=[0])
        fetch_resp_valid = RegArray(Bits(1), 1, initializer=[0])
        fetch_resp_addr = RegArray(Bits(32), 1, initializer=[0])
        fetch_resp_epoch = RegArray(Bits(8), 1, initializer=[0])

        on_branch = on_branch.optional(Bits(1)(0))
        branch_resolved = br_sm[0] & exec_br_valid[0] & ~fetch_resp_valid[0]
        redirect_addr = exec_br_taken[0].select(ex_bypass[0].bitcast(Bits(32)), pc_addr)
        branch_taken = branch_resolved & exec_br_taken[0]
        next_epoch = (fetch_epoch[0].bitcast(UInt(8)) + UInt(8)(1)).bitcast(Bits(8))
        next_br_sm = branch_taken.select(Bits(1)(0), on_branch | (br_sm[0] & ~branch_resolved))
        br_sm[0] = next_br_sm

        with Condition(branch_taken):
            fetch_epoch[0] = next_epoch

        should_fetch = ((~on_branch) & (~br_sm[0])) | branch_resolved


        new_cnt = ongoing[0] - (ex_valid.optional(Bits(1)(0))).select(Int(8)(1), Int(8)(0))
        to_fetch = branch_resolved.select(redirect_addr, pc_addr)
        issue_epoch = branch_taken.select(next_epoch, fetch_epoch[0])
        real_fetch = should_fetch & (ongoing[0] == Int(8)(0))
        log("on_br: {}         | br_sm: {}     | br_taken: {}      | fetch: {}      | ex_bypass: 0x{:05x} | ongoing: {} | epoch: 0x{:02x}",
             on_branch, br_sm[0], branch_taken, should_fetch, ex_bypass[0], ongoing[0], issue_epoch)
        icache.build(Bits(1)(0), real_fetch, to_fetch[2:2+depth_log-1].bitcast(Int(depth_log)), Bits(32)(0))
        log("on_br: {}         | de_by: {}     | fetch: {}      | addr: 0x{:05x} | new_cnt: {}",
            on_branch, ex_valid.optional(Bits(1)(0)), real_fetch, to_fetch, new_cnt)

        with Condition(fetch_resp_valid[0]):
            log(
                "dispatch        | addr: 0x{:08x} | epoch: 0x{:02x} | raw: 0x{:08x}",
                fetch_resp_addr[0],
                fetch_resp_epoch[0],
                icache.dout[0].bitcast(Bits(32)),
            )
            decoder.async_called(
                raw_inst=icache.dout[0].bitcast(Bits(32)),
                fetch_addr=fetch_resp_addr[0],
                epoch=fetch_resp_epoch[0],
            )

        with Condition(real_fetch):
            fetch_resp_valid[0] = Bits(1)(1)
            fetch_resp_addr[0] = to_fetch
            fetch_resp_epoch[0] = issue_epoch
            pc_reg[0] = (to_fetch.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
            ongoing[0] = new_cnt + Int(8)(1)
        
        with Condition(~real_fetch):
            fetch_resp_valid[0] = Bits(1)(0)
            pc_reg[0] = to_fetch
            ongoing[0] = new_cnt

class Onwrite(Downstream):
    
    def __init__(self):
        super().__init__()
        self.name = 'W1'

    @downstream.combinational
    def build(self, reg_onwrite: Array, exec_rd: Value, writeback_rd: Value):
        ex_rd = exec_rd.optional(Bits(5)(0))
        wb_rd = writeback_rd.optional(Bits(5)(0))
        ex_bit = (ex_rd != Bits(5)(0)).select(Bits(32)(1) << ex_rd, Bits(32)(0))
        wb_bit = (wb_rd != Bits(5)(0)).select(Bits(32)(1) << wb_rd, Bits(32)(0))

        # Clear completed writers first, then mark the new destination as busy.
        reg_onwrite[0] = (reg_onwrite[0] & ~wb_bit) | ex_bit
        log("ownning: {:02}      | releasing: {:02}| reg_onwrite[0]: {:08x}", ex_rd, wb_rd, reg_onwrite[0])

class MemUser(Module):
    def __init__(self):
        super().__init__(
            ports={}
        )
    @module.combinational
    def build(self,rdata:RegArray):
        width = rdata.scalar_ty.bits
        rdata = rdata[0].bitcast(Int(width))
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
        init_cache.build(we=Bits(1)(0), re=init_reg[0].bitcast(Bits(1)), wdata=Bits(32)(0), addr=Bits(5)(0))

        # Initialze offset at first cycle
        with Condition(init_reg[0]==UInt(1)(1)):
            user.async_called()
            init_reg[0] = UInt(1)(0)
        # Async_call after first cycle
        with Condition(init_reg[0] == UInt(1)(0)):
            d_call = fetcher.async_called()
        return init_cache

def build_cpu(depth_log):
    sys = SysBuilder('minor_cpu')

    with sys:
        # Data Types
        bits1   = Bits(1)
        bits5   = Bits(5)
        bits32  = Bits(32)

        user = MemUser()

        fetcher = Fetcher()
        pc_reg, pc_addr = fetcher.build()

        fetcher_impl = FetcherImpl()

        # Data Structures
        reg_file    = RegArray(bits32, 32)
        reg_onwrite = RegArray(bits32, 1)

        csr_file = RegArray(Bits(32), 16, initializer=[0]*16)

        exec_bypass_reg = RegArray(bits5, 1)
        exec_bypass_data = RegArray(bits32, 1)
        fetch_epoch = RegArray(Bits(8), 1, initializer=[0])

        mem_bypass_reg = RegArray(bits5, 1)
        mem_bypass_data = RegArray(bits32, 1)

        wb_bypass_reg = RegArray(bits5, 1)
        wb_bypass_data = RegArray(bits32, 1)
        exec_br_dest = RegArray(Bits(32), 1)
        exec_br_valid = RegArray(Bits(1), 1, initializer=[0])
        exec_br_taken = RegArray(Bits(1), 1, initializer=[0])
        d_br_buffer = RegArray(Bits(1), 1, initializer=[0])
        icache = SRAM(width=32, depth=1<<depth_log, init_file=f"{workspace}/workload.exe")
        icache.name = 'icache'

        writeback = WriteBack()
        wb_rd = writeback.build(reg_file = reg_file)

        memory_access = MemoryAccess()

        driver = Driver()
        init_cache = driver.build(fetcher, user)

        executor = Execution()
        offset_reg = user.build(init_cache.dout)

        exec_rd, ex_valid, dcache = executor.build(
            pc = pc_reg,
            exec_bypass_reg = exec_bypass_reg,
            exec_bypass_data = exec_bypass_data,
            reg_onwrite = reg_onwrite,
            mem_bypass_reg = mem_bypass_reg,
            mem_bypass_data = mem_bypass_data,
            wb_bypass_reg = wb_bypass_reg,
            wb_bypass_data = wb_bypass_data,
            offset_reg = offset_reg,
            fetch_epoch = fetch_epoch,
            rf = reg_file,
            csr_f = csr_file,
            memory = memory_access,
            data = f'{workspace}/workload.data',
            #writeback = writeback,
            depth_log = depth_log,
            exec_br_dest = exec_br_dest,
            exec_br_valid = exec_br_valid,
            exec_br_taken = exec_br_taken,


        )

        memory_access.build(
            writeback = writeback,
            mem_bypass_reg = mem_bypass_reg,
            mem_bypass_data=mem_bypass_data,
            wb_bypass_reg=wb_bypass_reg,
            wb_bypass_data=wb_bypass_data,
            rdata=dcache.dout,
        )

        decoder = Decoder()
        on_br = decoder.build(executor=executor)

        fetcher_impl.build(on_br, exec_br_dest, ex_valid, pc_reg,
                            pc_addr, decoder,
                              depth_log, d_br_buffer,
                              exec_br_valid, exec_br_taken, fetch_epoch, icache)

        onwrite_downstream = Onwrite()


        onwrite_downstream.build(
            reg_onwrite=reg_onwrite,
            exec_rd=exec_rd,
            writeback_rd=wb_rd,
        )
        '''RegArray exposing'''
        sys.expose_on_top(reg_file, kind='Output')
        sys.expose_on_top(reg_onwrite, kind='Output')
        sys.expose_on_top(csr_file, kind='Inout')
        sys.expose_on_top(pc_reg, kind='Output')


        '''Exprs exposing'''
        sys.expose_on_top(offset_reg, kind='Inout')
        sys.expose_on_top(ex_valid, kind='Output')




    print(sys)
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=5000000,
        idle_threshold=5000000,
        resource_base='',
        fifo_depth=4,
        enable_cache=False,
    )

    simulator_path, verilog_path = elaborate(sys, **conf)

    # Build the simulator binary once
    print("Building simulator binary...")
    simulator_binary = utils.build_simulator(simulator_path)
    print(f"Simulator binary built: {simulator_binary}")

    # Return the built system and relevant components
    return sys, simulator_binary, verilog_path


def extract_last_cycle(raw):
    last_cycle = 0
    for match in re.finditer(r'Cycle @([0-9]+(?:\.[0-9]+)?)', raw):
        last_cycle = max(last_cycle, int(float(match.group(1))))
    return last_cycle


def extract_kernel_cycles(raw):
    marker_cycles = []
    for line in raw.splitlines():
        if ('csr_addr: 0xb00' not in line) and ('imm: 0xb00' not in line):
            continue
        match = re.search(r'Cycle @([0-9]+(?:\.[0-9]+)?)', line)
        if match is None:
            continue
        cycle = int(float(match.group(1)))
        if not marker_cycles or marker_cycles[-1] != cycle:
            marker_cycles.append(cycle)
    if len(marker_cycles) >= 2:
        return marker_cycles[-1] - marker_cycles[0]
    return None


def measure_cycles(raw):
    kernel_cycles = extract_kernel_cycles(raw)
    if kernel_cycles is not None:
        return kernel_cycles, 'csr_b00'
    return extract_last_cycle(raw), 'full'


def run_cpu(sys, simulator_binary, verilog_path, workload='default', run_verilator_check=True):
    with sys:
        with open(f'{workspace}/workload.config') as f:
            raw = f.readline()
            raw = raw.replace('offset:', "'offset':").replace('data_offset:', "'data_offset':")
            offsets = eval(raw)
            value = hex(offsets['data_offset'])
            value = value[1:] if value[0] == '-' else value
            value = value[2:]
            open(f'{workspace}/workload.init', 'w').write(value)

    report = False
    sim_raw = ''

    if report:
        sim_raw = utils.run_simulator(binary_path=simulator_binary, offline=False)
        open(f'{workload}.log', 'w').write(sim_raw)
        #open(f'{workload}.sim.time', 'w').write(str(tt))
        raw = utils.run_verilator(verilog_path)
        open(f'{workload}.verilog.log', 'w').write(raw)
    else:
        sim_raw = utils.run_simulator(binary_path=simulator_binary)
        open('raw.log', 'w').write(sim_raw)
        check()
        if run_verilator_check:
            raw = utils.run_verilator(verilog_path)
            open('raw.log', 'w').write(raw)
            check()
        os.remove('raw.log')
    return measure_cycles(sim_raw)


def check():
    raw_path = 'raw.log'
    script = f'{workspace}/workload.sh'
    if os.path.exists(script):
        res = subprocess.run([script, raw_path, f'{workspace}/workload.data'])
        assert res.returncode == 0, f'Failed test: {res.returncode}'
        print('Test passed!!!')
        return

    script = f'{current_path}/../utils/find_pass.sh'
    res = subprocess.run([script, raw_path])
    if res.returncode == 0:
        print('Test passed!!!')
        return

    raw = open(raw_path).read()
    finished = ('ebreak | halt | ecall' in raw) or ('trap' in raw)
    assert finished, f'Failed test: {res.returncode}'
    print('Test passed!!!')

 
def cp_if_exists(src, dst, placeholder):
    if os.path.exists(src):
        shutil.copy(src, dst)
    elif placeholder:
        open(dst, 'w').write('')

def cp_required(src, dst):
    if not os.path.exists(src):
        raise FileNotFoundError(f'Missing required workload asset: {src}')
    shutil.copy(src, dst)

def init_workspace(base_path, case):
    if os.path.exists(f'{workspace}'):
        shutil.rmtree(f'{workspace}')
    os.mkdir(f'{workspace}')
    cp_required(f'{base_path}/{case}.exe', f'{workspace}/workload.exe')
    cp_if_exists(f'{base_path}/{case}.data', f'{workspace}/workload.data', True)
    cp_required(f'{base_path}/{case}.config', f'{workspace}/workload.config')
    cp_if_exists(f'{base_path}/{case}.sh', f'{workspace}/workload.sh', False)

def is_coremark_source_tree(path):
    required = [
        'core_list_join.c',
        'core_main.c',
        'core_matrix.c',
        'core_state.c',
        'core_util.c',
    ]
    return os.path.isdir(path) and all(os.path.exists(os.path.join(path, name)) for name in required)

def build_coremark_workload(coremark_src, workloads_path):
    script = os.path.join(workloads_path, 'build_coremark.sh')
    env = os.environ.copy()
    env['COREMARK_SRC'] = os.path.abspath(coremark_src)
    print(f'Packaging CoreMark from {env["COREMARK_SRC"]}')
    subprocess.run(['bash', script], check=True, env=env)
    return 'coremark'

def is_coremark_baremetal_source(path):
    return os.path.isfile(path) and os.path.basename(path) == 'coremark_baremetal.c'

def build_coremark_baremetal_workload(coremark_src, workloads_path):
    script = os.path.join(workloads_path, 'build_coremark_baremetal.sh')
    env = os.environ.copy()
    env['COREMARK_BAREMETAL_SRC'] = os.path.abspath(coremark_src)
    print(f'Packaging bare-metal CoreMark from {env["COREMARK_BAREMETAL_SRC"]}')
    subprocess.run(['bash', script], check=True, env=env)
    return 'coremark_baremetal'

def resolve_workload_case(workloads_path, spec):
    spec = WORKLOAD_ALIASES.get(spec, spec)
    packaged_exe = os.path.join(workloads_path, f'{spec}.exe')
    packaged_config = os.path.join(workloads_path, f'{spec}.config')
    if os.path.exists(packaged_exe) and os.path.exists(packaged_config):
        return spec

    abs_spec = os.path.abspath(spec)
    if is_coremark_source_tree(abs_spec):
        return build_coremark_workload(abs_spec, workloads_path)
    if is_coremark_baremetal_source(abs_spec):
        return build_coremark_baremetal_workload(abs_spec, workloads_path)

    if os.path.exists(abs_spec):
        if abs_spec.endswith('.ds'):
            raise ValueError(
                f'Unsupported workload path: {abs_spec}. '
                'This is DISC assembly, while examples/minor-cpu executes RV32I images. '
                'Use the packaged coremark/coremask workload or pass a CoreMark source tree.'
            )
        raise ValueError(
            f'Unsupported workload path: {abs_spec}. '
            'Currently only packaged workload names and CoreMark source trees are supported.'
        )

    return spec


def write_cycle_summary(rows):
    summary_path = f'{current_path}/kernel_cycles.csv'
    with open(summary_path, 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['workload', 'minor_cpu_cycles', 'cycle_source'])
        for workload, cycles, cycle_source in rows:
            writer.writerow([workload, cycles, cycle_source])
    return summary_path


def run_workloads(sys, simulator_binary, verilog_path, base_path, workloads, run_verilator_check):
    rows = []
    for workload in workloads:
        init_workspace(base_path, workload)
        cycles, cycle_source = run_cpu(
            sys,
            simulator_binary,
            verilog_path,
            workload,
            run_verilator_check,
        )
        rows.append((workload, cycles, cycle_source))
        print(f'{workload}: cycles={cycles} source={cycle_source}')
    return rows

def parse_args(argv):
    parser = argparse.ArgumentParser(
        description='Run the minor CPU on packaged workloads or a CoreMark source tree.'
    )
    parser.add_argument(
        '--sim-only',
        '--skip-verilator',
        dest='sim_only',
        action='store_true',
        help='Run the simulator pass only and skip the Verilator check.',
    )
    parser.add_argument(
        'workloads',
        nargs='*',
        help='Packaged workload names such as coremark/coremask, or a CoreMark source tree path such as /Users/zhonga/coremark.',
    )
    return parser.parse_args(argv)

if __name__ == '__main__':
    args = parse_args(py_sys.argv[1:])
    # Build the CPU Module only once
    sys, simulator_binary, verilog_path = build_cpu(depth_log=16)
    print("minor-CPU built successfully!")
    # Define workloads
    wl_path = f'{utils.repo_path()}/examples/minor-cpu/workloads'
    run_verilator_check = not args.sim_only
    if not args.workloads:
        workloads = [
            #'0to100',
            #'median',
            #'multiply',
            #'qsort',
            #'rsort',
            #'towers',
            #'vvadd',
        ]
        rows = run_workloads(sys, simulator_binary, verilog_path, wl_path, workloads, run_verilator_check)
        if rows:
            summary_path = write_cycle_summary(rows)
            print(f'Kernel cycle summary written to {summary_path}')
        print("minor-CPU workloads ran successfully!")

        # ========================================================================================
        # The same logic should be able to apply to the tests below, while the offsets&data_offsets should be changed accordingly.
        # Define test cases
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
            'rv32ui-p-lbu',
            'rv32ui-p-sb',
        ]
        tests = f'{utils.repo_path()}/examples/minor-cpu/unit-tests'
        # Iterate test cases
        for case in test_cases:
            # Copy test cases to tmp directory and rename to workload.
            init_workspace(tests, case)
            run_cpu(sys, simulator_binary, verilog_path, run_verilator_check=run_verilator_check)
        print("minor-CPU tests ran successfully!")
    else:
        # If user DID specify workloads, run exactly those, skipping default & tests:
        workloads = []
        for spec in args.workloads:
            wl = resolve_workload_case(wl_path, spec)
            workloads.append(wl)
        rows = run_workloads(sys, simulator_binary, verilog_path, wl_path, workloads, run_verilator_check)
        if rows:
            summary_path = write_cycle_summary(rows)
            print(f'Kernel cycle summary written to {summary_path}')
        print("Done running user-specified workload(s)!")
