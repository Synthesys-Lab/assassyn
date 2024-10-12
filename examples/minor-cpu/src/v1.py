''' A simplest single issue RISCV CPU, which has no operand buffer.
'''

import pytest

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

from opcodes import *
from decoder import *
from writeback import *
from memory_access import *
from utils import *

class Execution(Module):

    def __init__(self):
        super().__init__(
            ports={
                'opcode': Port(Bits(7)),
                'imm_value': Port(Bits(32)),
                'a_reg': Port(Bits(5)),
                'b_reg': Port(Bits(5)),
                'rd_reg': Port(Bits(5)),
            })
        self.name = "Executor"

    @module.combinational
    def build(
        self, 
        on_branch: Array, 
        pc: Array, 
        exec_bypass_reg: Array,
        exec_bypass_data: Array,
        mem_bypass_reg: Array,
        mem_bypass_data: Array,
        reg_onwrite: Array,
        rf: Array, 
        memory: Module, 
        writeback: Module,
        dcache: Module,):

        a_reg = self.a_reg.peek()
        b_reg = self.b_reg.peek()
        rd_reg = self.rd_reg.peek()

        on_write = reg_onwrite[0]

        a_valid = (((~(on_write >> self.a_reg.peek())) & Bits(32)(1)))[0:0] | \
            (exec_bypass_reg[0] == self.a_reg.peek()) | \
            (mem_bypass_reg[0] == self.a_reg.peek())
        b_valid = (((~(on_write >> self.b_reg.peek())) & Bits(32)(1)))[0:0] | \
            (exec_bypass_reg[0] == self.b_reg.peek()) | \
            (mem_bypass_reg[0] == self.b_reg.peek())

        rd_valid = (((~(on_write >> self.rd_reg.peek())) & Bits(32)(1)))[0:0]

        valid = a_valid & b_valid & rd_valid

        with Condition(~valid):
            log("scoreboard       | rs1-x{:02}:{:05}| rs2-x{:02}:{:05}| rd-x{:02}: {}", \
                a_reg, a_valid, \
                b_reg, b_valid, \
                rd_reg, rd_valid)

        wait_until(valid)

        opcode, imm_value, a_reg, b_reg, rd_reg = self.pop_all_ports(False)


        op_check = OpcodeChecker(opcode)
        op_check.check('lui', 'addi', 'add', 'lw', 'bne', 'ret', 'ebreak')

        is_lui    = op_check.lui
        is_addi   = op_check.addi
        is_add    = op_check.add
        is_lw     = op_check.lw
        is_bne    = op_check.bne
        is_ebreak = op_check.ebreak

        with Condition(is_ebreak):
            log('ebreak({:07b}) | halt', opcode)
            finish()

        # Instruction attributes
        uses_imm = is_addi | is_bne
        is_branch = is_bne

        a = (exec_bypass_reg[0] == a_reg).select(
            exec_bypass_data[0], 
            (mem_bypass_reg[0] == a_reg).select(mem_bypass_data[0], rf[a_reg])
        )
        b = (exec_bypass_reg[0] == b_reg).select(
            exec_bypass_data[0], 
            (mem_bypass_reg[0] == b_reg).select(mem_bypass_data[0], rf[b_reg])
        )
        
        rhs = uses_imm.select(imm_value, b)

        invoke_adder = is_add | is_addi | is_lw

        result = (a.bitcast(Int(32)) + rhs.bitcast(Int(32))).bitcast(Bits(32))
        result = (concat(invoke_adder, is_lui, is_branch)).select1hot(
            Bits(32)(0), imm_value, result
        )
        with Condition(invoke_adder):
            log("add              | a: {:08x}  | b:{:08x}    | res: {:08x}", a, rhs, result)

        produced_by_exec = is_lui | is_addi | is_add


        exec_bypass_reg[0] = produced_by_exec.select(rd_reg, Bits(5)(0))
        exec_bypass_data[0] = produced_by_exec.select(result, Bits(32)(0))

        with Condition(is_branch):
            on_branch[0] = Bits(1)(0)
            log("clear-br({:b})| on_branch = 0", opcode)
        
        with Condition(is_bne):
            delta = imm_value[0:12]
            delta = delta[12:12].select(Bits(19)(0x7ffff), Bits(19)(0)).concat(delta).bitcast(Int(32))
            log('delta: {:x}', delta)
            dest_pc = (pc[0].bitcast(Int(32)) - Int(32)(8) + delta).bitcast(Bits(32))
            new_pc = (pc[0].bitcast(Int(32)) - Int(32)(4)).bitcast(Bits(32))
            br_dest = (a != b).select(dest_pc, new_pc)
            log("bne({:b})     | {} != {} | to {} | else {}", opcode, a, b, dest_pc, new_pc)
            pc[0] = br_dest

        is_memory = is_lw
        is_memory_read = is_lw

        request_addr = is_memory.select(result[2:10].bitcast(Int(9)), Int(9)(0))

        mem_bypass_reg[0] = is_memory_read.select(rd_reg, Bits(5)(0))

        with Condition(is_memory):
            log("mem-read         | addr: {:x} | lineno: {:x}", result, request_addr)


        dcache.build(we=Int(1)(0), re=is_memory_read, wdata=a, addr=request_addr, user=memory)
        dcache.bound.async_called()
        wb = writeback.bind(opcode = opcode, result = result, rd = rd_reg)

        with Condition(rd_reg != Bits(5)(0)):
            return_rd = rd_reg
            log("with-rd({:07b}) | own x{:02}", opcode, rd_reg)

        return wb, return_rd

class Decoder(Module):
    
    def __init__(self):
        super().__init__(ports={
            'rdata': Port(Bits(32))
        })
        self.name = 'Decoder'

    @module.combinational
    def build(self, pc: Array, on_branch: Array, executor: Module):
        inst = self.pop_all_ports(False)
        with Condition(~on_branch[0]):
            signals = decode_logic(inst)

            with Condition(signals.is_branch):
                on_branch[0] = Bits(1)(1)

            executor.async_called(
                opcode = inst[0:6],
                imm_value = signals.imm_value,
                a_reg = signals.rs1_reg,
                b_reg = signals.rs2_reg,
                rd_reg = signals.rd_reg)

        with Condition(on_branch[0]):
            log("on a branch, stall decoding, pc freeze at 0x{:x}", pc[0])

class Fetcher(Module):
    
    def __init__(self):
        super().__init__(ports={})
        self.name = 'Fetcher'

    @module.combinational
    def build(self, decoder: Decoder, pc: Array, on_branch: Array, icache: SRAM):
        to_fetch = pc[0][2:10].bitcast(Int(9))
        icache.build(Bits(1)(0), ~on_branch[0], to_fetch, Bits(32)(0), decoder)
        with Condition(~on_branch[0]):
            log("fetching         | *inst[0x{:x}]", pc[0])
            pc[0] = (pc[0].bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
            # Call the decoder
            icache.bound.async_called()

        with Condition(on_branch[0]):
            log("fetching         | on branch, pc freeze at 0x{:x}", pc[0])

class OnwriteDS(Downstream):
    
    def __init__(self):
        super().__init__()
        self.name = 'Onwrite'

    @downstream.combinational
    def build(self, reg_onwrite: Array, exec_rd: Value, writeback_rd: Value):
        ex_rd = exec_rd.optional(Bits(5)(0))
        wb_rd = writeback_rd.optional(Bits(5)(0))

        log("scoreboard       | ownning: x{:02} | releasing: x{:02}", ex_rd, wb_rd)

        reg_onwrite[0] = reg_onwrite[0] ^ \
                        (Bits(32)(1) << wb_rd) ^ \
                        (Bits(32)(1) << ex_rd)

class Driver(Module):
    
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, fetcher: Module):
        fetcher.async_called()

def main():
    sys = SysBuilder('cpu_v1')

    with sys:
        # Data Types
        bits1   = Bits(1)
        bits5   = Bits(5)
        bits32  = Bits(32)

        icache = SRAM(width=32, depth=512, init_file='0to100.exe')
        icache.name = 'icache'
        dcache = SRAM(width=32, depth=512, init_file='0to100.data')
        dcache.name = 'dcache'

        # Data Structures
        pc          = RegArray(bits32, 1)
        on_branch   = RegArray(bits1, 1)
        reg_file    = RegArray(bits32, 32)
        reg_onwrite = RegArray(bits32, 1)

        exec_bypass_reg = RegArray(bits5, 1)
        exec_bypass_data = RegArray(bits32, 1)

        mem_bypass_reg = RegArray(bits5, 1)
        mem_bypass_data = RegArray(bits32, 1)

        writeback = WriteBack()
        wb_rd = writeback.build(reg_file = reg_file)

        memory_access = MemoryAccess()

        executor = Execution()
        wb, exec_rd = executor.build(
            pc = pc,
            on_branch=on_branch,
            exec_bypass_reg = exec_bypass_reg,
            reg_onwrite = reg_onwrite,
            exec_bypass_data = exec_bypass_data,
            mem_bypass_reg = mem_bypass_reg,
            mem_bypass_data = mem_bypass_data,
            rf = reg_file,
            memory = memory_access,
            writeback = writeback,
            dcache = dcache
        )

        memory_access.build(
            writeback = wb, 
            mem_bypass_reg = mem_bypass_reg, 
            mem_bypass_data=mem_bypass_data
        )

        decoder = Decoder()
        decoder.build(pc = pc, on_branch = on_branch, executor = executor)

        onwrite_downstream = OnwriteDS()
    
        fetcher = Fetcher()
        fetcher.build(decoder, pc, on_branch, icache)

        driver = Driver()
        driver.build(fetcher)

        onwrite_downstream.build(
            reg_onwrite=reg_onwrite,
            exec_rd=exec_rd,
            writeback_rd=wb_rd,
        )

    print(sys)
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=1500,
        idle_threshold=1500,
        resource_base=f'{utils.repo_path()}/examples/minor-cpu/resource'
    )

    simulator_path, verilog_path = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    raw = utils.run_verilator(verilog_path)
    check(raw)

if __name__ == '__main__':
    main()
