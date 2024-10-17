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

offset = None
data_offset = None

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
        pc: Array, 
        exec_bypass_reg: Array,
        exec_bypass_data: Array,
        mem_bypass_reg: Array,
        mem_bypass_data: Array,
        reg_onwrite: Array,
        rf: Array, 
        memory: Module, 
        writeback: Module,
        data: str):

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
        uses_imm = is_addi | is_bne | is_lw
        is_branch = is_bne

        a = (exec_bypass_reg[0] == a_reg).select(
            exec_bypass_data[0], 
            (mem_bypass_reg[0] == a_reg).select(mem_bypass_data[0], rf[a_reg])
        )
        b = (exec_bypass_reg[0] == b_reg).select(
            exec_bypass_data[0], 
            (mem_bypass_reg[0] == b_reg).select(mem_bypass_data[0], rf[b_reg])
        )

        # log('mem_bypass_reg: x{:02} | mem_bypass_data: {:08x}', mem_bypass_reg[0], mem_bypass_data[0])
        # log('exe_bypass_reg: x{:02} | exe_bypass_data: {:08x}', exec_bypass_reg[0], exec_bypass_data[0])

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

        with Condition(is_bne):
            delta = imm_value[0:12]
            delta = delta[12:12].select(Bits(19)(0x7ffff), Bits(19)(0)).concat(delta).bitcast(Int(32))
            log('delta: {:x}', delta)
            br_pc = (pc[0].bitcast(Int(32)) - Int(32)(4) + delta).bitcast(Bits(32))
            nxt_pc = pc[0]
            br_dest = (a != b).select(br_pc, nxt_pc)
            log("bne({:b})     | {} != {} | to {} | else {}", opcode, a, b, br_pc, nxt_pc)
            br_sm = RegArray(Bits(1), 1)
            br_sm[0] = Bits(1)(0)

        is_memory = is_lw
        is_memory_read = is_lw

        addr = (result.bitcast(Int(32)) - offset - data_offset).bitcast(Bits(32))

        request_addr = is_memory.select(addr[2:10].bitcast(Int(9)), Int(9)(0))

        mem_bypass_reg[0] = is_memory_read.select(rd_reg, Bits(5)(0))

        with Condition(is_memory):
            log("mem-read         | addr: 0x{:x} | lineno: 0x{:x}", result, request_addr)
    

        dcache = SRAM(width=32, depth=512, init_file=data)
        dcache.name = 'dcache'
        dcache.build(we=Int(1)(0), re=is_memory_read, wdata=a, addr=request_addr, user=memory)
        dcache.bound.async_called()
        wb = writeback.bind(opcode = opcode, result = result, rd = rd_reg)

        with Condition(rd_reg != Bits(5)(0)):
            return_rd = rd_reg
            log("with-rd({:07b}) | own x{:02}", opcode, rd_reg)

        return br_sm, br_dest, wb, return_rd

class Decoder(Module):
    
    def __init__(self):
        super().__init__(ports={
            'rdata': Port(Bits(32))
        })
        self.name = 'Decoder'

    @module.combinational
    def build(self, executor: Module, br_sm: Array):

        inst = self.pop_all_ports(False)

        signals = decode_logic(inst)

        br_sm[0] = signals.is_branch

        executor.async_called(
            opcode = inst[0:6],
            imm_value = signals.imm_value,
            a_reg = signals.rs1_reg,
            b_reg = signals.rs2_reg,
            rd_reg = signals.rd_reg)

        return signals.is_branch

class Fetcher(Module):
    
    def __init__(self):
        super().__init__(ports={})
        self.name = 'Fetcher'

    @module.combinational
    def build(self):
        pc_reg = RegArray(Bits(32), 1)
        addr = pc_reg[0]
        return pc_reg, addr

class FetcherImpl(Downstream):

    def __init__(self):
        super().__init__()
        self.name = 'FetcherImpl'

    @downstream.combinational
    def build(self,
              on_branch: Value,
              br_sm: Array,
              ex_bypass: Value,
              pc_reg: Value,
              pc_addr: Value,
              decoder: Decoder,
              data: str):
        on_branch = on_branch.optional(Bits(1)(0)) | br_sm[0]
        should_fetch = ~on_branch | ex_bypass.valid()
        to_fetch = ex_bypass.optional(pc_addr)
        icache = SRAM(width=32, depth=512, init_file=data)
        icache.name = 'icache'
        icache.build(Bits(1)(0), should_fetch, to_fetch[2:10].bitcast(Int(9)), Bits(32)(0), decoder)
        log("fetcher          | on_br: {} | ex_by: {} | should_fetch: {} | fetch: 0x{:x}",
            on_branch, ex_bypass.valid(), should_fetch, to_fetch)
        with Condition(should_fetch):
            icache.bound.async_called()
            pc_reg[0] = (to_fetch.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))

class Onwrite(Downstream):
    
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

def run_cpu(workload):
    sys = SysBuilder('minor_cpu')

    resource_base = f'{utils.repo_path()}/examples/minor-cpu/resource'

    with sys:

        with open(f'{resource_base}/{workload}.config') as f:
            global offset, data_offset
            raw = f.readline()
            raw = raw.replace('offset:', "'offset':").replace('data_offset:', "'data_offset':")
            offsets = eval(raw)
            print(offsets)
            offset = offsets['offset']
            data_offset = offsets['data_offset']
            offset = Int(32)(offset)
            data_offset = Int(32)(data_offset)

        # Data Types
        bits1   = Bits(1)
        bits5   = Bits(5)
        bits32  = Bits(32)

        fetcher = Fetcher()
        pc_reg, pc_addr = fetcher.build()

        fetcher_impl = FetcherImpl()

        # Data Structures
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

        data_init = f'{workload}.data' if os.path.exists(f'{resource_base}/{workload}.data') else None

        br_sm, ex_bypass, wb, exec_rd = executor.build(
            pc = pc_reg,
            exec_bypass_reg = exec_bypass_reg,
            reg_onwrite = reg_onwrite,
            exec_bypass_data = exec_bypass_data,
            mem_bypass_reg = mem_bypass_reg,
            mem_bypass_data = mem_bypass_data,
            rf = reg_file,
            memory = memory_access,
            writeback = writeback,
            data = data_init
        )

        memory_access.build(
            writeback = wb, 
            mem_bypass_reg = mem_bypass_reg, 
            mem_bypass_data=mem_bypass_data
        )

        decoder = Decoder()
        on_br = decoder.build(executor=executor, br_sm=br_sm)

        fetcher_impl.build(on_br, br_sm, ex_bypass, pc_reg, pc_addr, decoder, f'{workload}.exe')

        onwrite_downstream = Onwrite()
    
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
        resource_base=resource_base
    )

    simulator_path, verilog_path = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    check(raw)

    raw = utils.run_verilator(verilog_path)
    check(raw)

if __name__ == '__main__':
    run_cpu('0to100')
