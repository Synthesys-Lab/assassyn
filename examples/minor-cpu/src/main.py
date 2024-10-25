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
        csr_f: Array,
        memory: Module, 
        writeback: Module,
        data: str,
        depth_log: int):

        
        csr_id = Bits(4)(0)
        
 
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



        raw_id = [(3860, 9), (773, 1) ,(1860, 15) , (384,10) , (944 , 11) , (928 , 12) , (772 , 4 ) , (770 ,13),(771,14),(768,8) ,(833,2)]
        #mtvec 1 mepc 2 mcause 3 mie 4 mip 5 mtval 6 mscratc 7 mstatus 8 mhartid 9 satp 10 pmpaddr0 11  pmpcfg0 12 medeleg 13 mideleg 14 unkonwn 15

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


        signals, fetch_addr = self.pop_all_ports(False)

        # TODO(@were): This is a hack to avoid post wait_until checks.
        rd = signals.rd

        is_ebreak = signals.rs1_valid & signals.imm_valid & ((signals.imm == Bits(32)(1))|(signals.imm == Bits(32)(0))) & (signals.alu == Bits(16)(0))
        with Condition(is_ebreak):
            log('ebreak | halt | ecall')
            finish()

        # Instruction attributes

        def bypass(bypass_reg, bypass_data, idx, value):
            return (bypass_reg[0] == idx).select(bypass_data[0], value)

        a = bypass(exec_bypass_reg, exec_bypass_data, rs1, rf[rs1])
        a = bypass(mem_bypass_reg, mem_bypass_data, rs1, a)
        a = (rs1 == Bits(5)(0)).select(Bits(32)(0), a)
        a = signals.csr_write.select(Bits(32)(0), a)

        b = bypass(exec_bypass_reg, exec_bypass_data, rs2, rf[rs2])
        b = bypass(mem_bypass_reg, mem_bypass_data, rs2, b)
        b = (rs2 == Bits(5)(0)).select(Bits(32)(0), b)
        b = is_csr.select(csr_f[csr_id], b)
        

        # log('mem_bypass.reg: x{:02} | .data: {:08x}', mem_bypass_reg[0], mem_bypass_data[0])
        # log('exe_bypass.reg: x{:02} | .data: {:08x}', exec_bypass_reg[0], exec_bypass_data[0])

        # TODO: To support `auipc`, is_branch will be separated into `is_branch` and `is_pc_calc`.
        alu_a = (signals.is_offset_br | signals.is_pc_calc).select(fetch_addr, a)
        alu_b = signals.imm_valid.select(signals.imm, b)

        results = [Bits(32)(0)] * RV32I_ALU.CNT

        adder_result = (alu_a.bitcast(Int(32)) + alu_b.bitcast(Int(32))).bitcast(Bits(32))
        le_result = (a.bitcast(Int(32)) < b.bitcast(Int(32))).select(Bits(32)(1), Bits(32)(0))
        eq_result = (a == b).select(Bits(32)(1), Bits(32)(0))
        leu_result = (Bits(1)(0).concat(a) < Bits(1)(0).concat(b ) ).select(Bits(32)(1), Bits(32)(0))
        alu_b_shift_bits = alu_b[4:4].select( concat(Int(27)(-1) , alu_b[0:4]).bitcast(Int(32)), concat(Bits(27)(0),  alu_b[0:4]).bitcast(Int(32)) )
        sra_signed_result = a[31:31].select( (a >> alu_b[0:4]) | ~((Int(32)(1) << (Int(32)(32) - alu_b_shift_bits )   ) - Int(32)(1)) , (a >> alu_b[0:4]))
        sub_result = (a.bitcast(Int(32)) - b.bitcast(Int(32))).bitcast(Bits(32))

        results[RV32I_ALU.ALU_ADD] = adder_result
        results[RV32I_ALU.ALU_SUB] = sub_result
        results[RV32I_ALU.ALU_CMP_LT] = le_result
        results[RV32I_ALU.ALU_CMP_EQ] = eq_result
        results[RV32I_ALU.ALU_CMP_LTU] = leu_result
        results[RV32I_ALU.ALU_XOR] = a ^ b
        results[RV32I_ALU.ALU_OR] = a | b
        results[RV32I_ALU.ALU_AND] = a & alu_b
        results[RV32I_ALU.ALU_TRUE] = Bits(32)(1)
        results[RV32I_ALU.ALU_SLL] = a << alu_b[0:4]
        results[RV32I_ALU.ALU_SRA] = a >> sra_signed_result 
        results[RV32I_ALU.ALU_SRA_U] = a >> alu_b[0:4]

        # TODO: Fix this bullshit.
        alu = signals.alu
        result = alu.select1hot(*results)

        log('is_offset_br: {}  | is_pc_calc: {} |', signals.is_offset_br, signals.is_pc_calc)
        log("0x{:08x}       | a: {:08x}  | b: {:08x}   | imm: {:08x} | result: {:08x}", alu, a, b, signals.imm, result)
        log("0x{:08x}       |a.a:{:08x}  |a.b:{:08x}   | res: {:08x} |", alu, alu_a, alu_b, result)

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

        with Condition(memory_read):
            mem_bypass_reg[0] = memory_read.select(rd, Bits(5)(0))
            log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)

        with Condition(memory_write):
            log("mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x}", result, request_addr, a)

        dcache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        dcache.name = 'dcache'
        dcache.build(we=memory_write, re=memory_read, wdata=b, addr=request_addr, user=memory)
        dcache.bound.async_called()
        wb = writeback.bind(is_memory_read = memory_read,
                            result = signals.link_pc.select(pc[0], result),
                            rd = rd,
                            is_csr = signals.csr_write,
                            csr_id = csr_id,
                            csr_new = csr_new,
                            mem_ext = signals.mem_ext)

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

        csr_file = RegArray(Bits(32), 16, initializer=[0]*16)

        exec_bypass_reg = RegArray(bits5, 1)
        exec_bypass_data = RegArray(bits32, 1)

        mem_bypass_reg = RegArray(bits5, 1)
        mem_bypass_data = RegArray(bits32, 1)

        writeback = WriteBack()
        wb_rd = writeback.build(reg_file = reg_file , csr_file = csr_file)

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
            csr_f = csr_file,
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
        sim_threshold=100000,
        idle_threshold=100000,
        resource_base=resource_base
    )

    simulator_path, verilog_path = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    open('raw.log', 'w').write(raw)
    test = f'{resource_base}/find_pass.sh'
    check(resource_base, workload)

    raw = utils.run_verilator(verilog_path)
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
        # 'multiply',
    ]
    for wl in workloads:
        run_cpu(wl_path, wl, 12)

    test_cases = [
        'rv32ui-p-add',
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
        #'rv32ui-p-lbu',
        #'rv32ui-p-lui',
        #'rv32ui-p-lw',
        #'rv32ui-p-sub',
        #'rv32ui-p-sw',
        #'rv32ui-p-or',
        #'rv32ui-p-ori',
    ]

    tests = f'{utils.repo_path()}/examples/minor-cpu/unit-tests'

    for case in test_cases:
        run_cpu(tests, case, 9)

