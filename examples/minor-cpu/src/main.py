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

offset = None
data_offset = None

class Execution(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'signals': Port(deocder_signals),
                'fetch_addr': Port(Bits(32)),
            })
        self.name = "E"

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
        data: str,
        depth_log: int):

        signals = self.signals.peek()

        rs1 = signals.rs1
        rs2 = signals.rs2
        rd = signals.rd

        on_write = reg_onwrite[0]

        a_valid = (~(on_write >> rs1))[0:0] | \
                  (exec_bypass_reg[0] == rs1) | \
                  (mem_bypass_reg[0] == rs1) | \
                  ~signals.rs1_valid

        b_valid = (~(on_write >> rs2))[0:0] | \
                  (exec_bypass_reg[0] == rs2) | \
                  (mem_bypass_reg[0] == rs2) | \
                  ~signals.rs2_valid

        rd_valid = (~(on_write >> rd))[0:0]

        valid = a_valid & b_valid & rd_valid

        with Condition(~valid):
            log("rs1-x{:02}: {}       | rs2-x{:02}: {}   | rd-x{:02}: {}", \
                rs1, a_valid, rs2, b_valid, rd, rd_valid)

        wait_until(valid)

        signals, fetch_addr = self.pop_all_ports(False)

        # TODO(@were): This is a hack to avoid post wait_until checks.
        rd = signals.rd

        is_ebreak = signals.rs1_valid & signals.imm_valid & (signals.imm == Bits(32)(1)) & (signals.alu == Bits(16)(0))
        with Condition(is_ebreak):
            log('ebreak | halt')
            finish()

        # Instruction attributes

        def bypass(bypass_reg, bypass_data, idx, value):
            return (bypass_reg[0] == idx).select(bypass_data[0], value)

        a = bypass(exec_bypass_reg, exec_bypass_data, rs1, rf[rs1])
        a = bypass(mem_bypass_reg, mem_bypass_data, rs1, a)
        a = (rs1 == Bits(5)(0)).select(Bits(32)(0), a)

        b = bypass(exec_bypass_reg, exec_bypass_data, rs2, rf[rs2])
        b = bypass(mem_bypass_reg, mem_bypass_data, rs2, b)
        b = (rs2 == Bits(5)(0)).select(Bits(32)(0), b)

        # log('mem_bypass.reg: x{:02} | .data: {:08x}', mem_bypass_reg[0], mem_bypass_data[0])
        # log('exe_bypass.reg: x{:02} | .data: {:08x}', exec_bypass_reg[0], exec_bypass_data[0])

        # TODO: To support `auipc`, is_branch will be separated into `is_branch` and `is_pc_calc`.
        alu_a = signals.is_branch.select(fetch_addr, a)
        alu_b = signals.imm_valid.select(signals.imm, b)

        results = [Bits(32)(0)] * RV32I_ALU.CNT

        adder_result = (alu_a.bitcast(Int(32)) + alu_b.bitcast(Int(32))).bitcast(Bits(32))
        le_result = (a.bitcast(Int(32)) < b.bitcast(Int(32))).select(Bits(32)(1), Bits(32)(0))
        eq_result = (a == b).select(Bits(32)(1), Bits(32)(0))

        results[RV32I_ALU.ALU_ADD] = adder_result
        results[RV32I_ALU.ALU_CMP_LT] = le_result
        results[RV32I_ALU.ALU_CMP_EQ] = eq_result
        results[RV32I_ALU.ALU_XOR] = a ^ b
        results[RV32I_ALU.ALU_OR] = a | b
        results[RV32I_ALU.ALU_AND] = a & b
        results[RV32I_ALU.ALU_TRUE] = Bits(32)(1)

        # TODO: Fix this bullshit.
        alu = signals.alu
        result = alu.select1hot(*results)

        log("0x{:08x}       | a: {:08x}  | b: {:08x}   | imm: {:08x} | result: {:08x}", alu, a, b, signals.imm, result)

        condition = signals.cond.select1hot(*results)
        condition = signals.flip.select(~condition, condition)

        memory_read = signals.memory[0:0]
        memory_write = signals.memory[1:1]

        # TODO: Make this stricter later.
        produced_by_exec = ~memory_read & (rd != Bits(5)(0))
        exec_bypass_reg[0] = produced_by_exec.select(rd, Bits(5)(0))
        exec_bypass_data[0] = produced_by_exec.select(result, Bits(32)(0))

        with Condition(signals.is_branch):
            br_dest = condition[0:0].select(result, pc[0])
            log("condition: {}.a.b | a: {:08x}  | b: {:08x}   |", condition[0:0], result, pc[0])
            br_sm = RegArray(Bits(1), 1)
            br_sm[0] = Bits(1)(0)

        is_memory = memory_read | memory_write

        # This `is_memory` hack is to evade rust's overflow check.
        addr = (result.bitcast(UInt(32)) - is_memory.select(data_offset, UInt(32)(0))).bitcast(Bits(32))
        request_addr = is_memory.select(addr[2:2+depth_log-1].bitcast(Int(depth_log)), Int(depth_log)(0))

        with Condition(is_memory):
            mem_bypass_reg[0] = memory_read.select(rd, Bits(5)(0))
            log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)

        dcache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        dcache.name = 'dcache'
        dcache.build(we=memory_write, re=memory_read, wdata=a, addr=request_addr, user=memory)
        dcache.bound.async_called()
        wb = writeback.bind(is_memory_read = memory_read, result = result, rd = rd)

        with Condition(rd != Bits(5)(0)):
            log("own x{:02}          |", rd)

        return br_sm, br_dest, wb, rd 

class Decoder(Module):
    
    def __init__(self):
        super().__init__(ports={
            'rdata': Port(Bits(32)),
            'fetch_addr': Port(Bits(32)),
        })
        self.name = 'D'

    @module.combinational
    def build(self, executor: Module, br_sm: Array):
        inst, fetch_addr = self.pop_all_ports(False)

        signals = decode_logic(inst)
        br_sm[0] = signals.is_branch

        executor.async_called(signals=signals, fetch_addr=fetch_addr)

        return signals.is_branch

class Fetcher(Module):
    
    def __init__(self):
        super().__init__(ports={})
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
              br_sm: Array,
              ex_bypass: Value,
              pc_reg: Value,
              pc_addr: Value,
              decoder: Decoder,
              data: str,
              depth_log: int):
        on_branch = on_branch.optional(Bits(1)(0)) | br_sm[0]
        should_fetch = ~on_branch | ex_bypass.valid()
        to_fetch = ex_bypass.optional(pc_addr)
        icache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        icache.name = 'icache'
        icache.build(Bits(1)(0), should_fetch, to_fetch[2:2+depth_log-1].bitcast(Int(depth_log)), Bits(32)(0), decoder)
        log("on_br: {}         | ex_by: {}     | fetch: {}      | addr: 0x{:05x} |",
            on_branch, ex_bypass.valid(), should_fetch, to_fetch)
        with Condition(should_fetch):
            icache.bound.async_called(fetch_addr=to_fetch)
            pc_reg[0] = (to_fetch.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))

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
        log("ownning: {:02}      | releasing: {:02}|", ex_rd, wb_rd)
        reg_onwrite[0] = reg_onwrite[0] ^ ex_bit ^ wb_bit

class Driver(Module):
    
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, fetcher: Module):
        fetcher.async_called()

def run_cpu(resource_base, workload, depth_log):
    sys = SysBuilder('minor_cpu')

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
            exec_bypass_data = exec_bypass_data,
            reg_onwrite = reg_onwrite,
            mem_bypass_reg = mem_bypass_reg,
            mem_bypass_data = mem_bypass_data,
            rf = reg_file,
            memory = memory_access,
            writeback = writeback,
            data = data_init,
            depth_log = depth_log
        )

        memory_access.build(
            writeback = wb, 
            mem_bypass_reg = mem_bypass_reg, 
            mem_bypass_data=mem_bypass_data
        )

        decoder = Decoder()
        on_br = decoder.build(executor=executor, br_sm=br_sm)

        fetcher_impl.build(on_br, br_sm, ex_bypass, pc_reg, pc_addr, decoder, f'{workload}.exe', depth_log)

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
    open('raw.log', 'w').write(raw)
    test = f'{resource_base}/{workload}.sh'
    subprocess.run([test, 'raw.log', f'{resource_base}/{workload}.data'])

    raw = utils.run_verilator(verilog_path)
    open('raw.log', 'w').write(raw)
    test = f'{resource_base}/{workload}.sh'
    subprocess.run([test, 'raw.log', f'{resource_base}/{workload}.data'])

    os.remove('raw.log')

if __name__ == '__main__':
    workloads = f'{utils.repo_path()}/examples/minor-cpu/workloads'
    run_cpu(workloads, '0to100', 9)

    # tests = f'{utils.repo_path()}/examples/minor-cpu/unit-tests'
    # run_cpu(tests, 'rv32ui-p-add')
    
    # workloads = f'{utils.repo_path()}/examples/minor-cpu/workloads'
    # run_cpu(workloads, 'multiply', 12)
