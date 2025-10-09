''' A simplest single issue RISCV CPU, which has no operand buffer.
'''
import os
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

class Execution(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'signals': Port(deocder_signals),
                'fetch_addr': Port(Bits(32)),
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
        rf: Array,
        csr_f: Array,
        memory: Module,
        data: str,
        depth_log: int,
        exec_br_dest: Array
        ):

        csr_id = Bits(4)(0)
 
        signals = self.signals.peek()

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

        with Condition(~valid):
            log("pc: 0x{:08x}   | rs1-x{:02}: {}       | rs2-x{:02}: {}   | rd-x{:02}: {} | backlogged", \
                self.fetch_addr.peek(), rs1, a_valid, rs2, b_valid, rd, rd_valid)

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


        signals, fetch_addr = self.pop_all_ports(False)
        

        # TODO(@were): This is a hack to avoid post wait_until checks.
        rd = signals.rd

        is_ebreak = signals.rs1_valid & signals.imm_valid & \
                    ((signals.imm == Bits(32)(1)) | (signals.imm == Bits(32)(0))) & \
                    (signals.alu == Bits(16)(1<<RV32I_ALU.ALU_NONE))        


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

        # Instruction attributes

        def bypass(bypass_reg, bypass_data, idx, value):
            return (bypass_reg[0] == idx).select(bypass_data[0], value)

        a = bypass(mem_bypass_reg, mem_bypass_data, rs1, rf[rs1])
        a = bypass(exec_bypass_reg, exec_bypass_data, rs1, a)
        a = (rs1 == Bits(5)(0)).select(Bits(32)(0), a)
        a = signals.csr_write.select(Bits(32)(0), a)
        a = (~a_valid).select( wb_bypass_data[0], a)

        b = bypass(mem_bypass_reg, mem_bypass_data, rs2, rf[rs2])
        b = bypass(exec_bypass_reg, exec_bypass_data, rs2, b)
        b = (rs2 == Bits(5)(0)).select(Bits(32)(0), b)
        b = is_csr.select(csr_f[csr_id], b)
        b = (~b_valid).select( wb_bypass_data[0], b)
        

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

        log('pc: 0x{:08x}   |is_offset_br: {}| is_pc_calc: {}|', fetch_addr, signals.is_offset_br, signals.is_pc_calc)
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

  
        pc0 = (fetch_addr.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
        with Condition(signals.is_branch):
            exec_br_dest[0] = condition[0:0].select(result, pc0)
            log("condition: {}.a.b | a: {:08x}  | b: {:08x}   |", condition[0:0], result, pc0)
            
        #exec_br_jumped[0] = signals.is_branch.select(condition[0:0], Bits(1)(0))
        exec_br_jump = signals.is_branch.select(condition[0:0], Bits(1)(0))

        is_memory = memory_read | memory_write

        # This `is_memory` hack is to evade rust's overflow check.
        addr = (result.bitcast(UInt(32)) - is_memory.select(offset_reg[0].bitcast(UInt(32)), UInt(32)(0))).bitcast(Bits(32))
        request_addr = is_memory.select(addr[2:2+depth_log-1].bitcast(UInt(depth_log)), UInt(depth_log)(0))

        with Condition(memory_read):
            log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)

        with Condition(memory_write):
            log("mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} | wdada: 0x{:08x}", result, request_addr, a, b)

        dcache = SRAM(width=32, depth=1<<depth_log, init_file=data)
        dcache.name = 'dcache'
        dcache.build(we=memory_write, re=memory_read, wdata=b, addr=request_addr)
        bound = memory.bind(rd = rd,result = signals.link_pc.select(pc0, result), mem_ext = signals.mem_ext,is_mem_read = memory_read)
        bound.async_called()
        with Condition(signals.csr_write):
            csr_f[csr_id] = csr_new

        with Condition(rd != Bits(5)(0)):
            log("own x{:02}          |", rd)

        return  rd, ex_valid ,exec_br_jump,dcache

class Decoder(Module):
    
    def __init__(self):
        super().__init__(ports={
            'fetch_addr': Port(Bits(32)),
        })
        self.name = 'D'

    @module.combinational
    def build(self, executor: Module,rdata:RegArray):
        fetch_addr = self.pop_all_ports(False)
        inst = rdata[0].bitcast(Bits(32))

        log("raw: 0x{:08x}  | addr: 0x{:05x} |", inst, fetch_addr)

        signals = decode_logic(inst)
 
        
        e_call = executor.async_called(signals=signals, fetch_addr=fetch_addr)
        e_call.bind.set_fifo_depth(signals=2, fetch_addr=2)

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
              br_jump: Array,
              br_no_jump: Array,
              exec_br_jump: Value,
              icache: SRAM,

              ):

        ongoing = RegArray(Int(8), 1, initializer=[0])

        on_branch = on_branch.optional(Bits(1)(0))
        br_sm[0] = on_branch

        br_jump[0] = exec_br_jump.optional(Bits(1)(0))
        br_no_jump[0] = ~ br_jump[0]

        fetch_valid=RegArray(Bits(1), 1, initializer=[1])

        with Condition(br_sm[0]):
            fetch_valid[0] = ex_valid.optional(Bits(1)(0)).select(Bits(1)(1), Bits(1)(0))
        with Condition(~fetch_valid[0]):
            fetch_valid[0] = Bits(1)(1)

        should_fetch =  (~ on_branch) & (~ br_sm[0] ) & fetch_valid[0]


        jump_flag = br_jump[0] & br_no_jump[0]


        new_cnt = ongoing[0] - (ex_valid.optional(Bits(1)(0))).select(Int(8)(1), Int(8)(0))
        to_fetch = Bits(32)(0)
        #to_fetch = should_fetch.select(pc_addr, to_fetch)
        to_fetch = (jump_flag).select(ex_bypass[0].bitcast(Bits(32)), pc_addr)
        real_fetch = (should_fetch  )& (new_cnt < Int(8)(3))
        log("on_br: {}         | br_sm: {}     | br_jump: {}      | fetch: {}      | ex_bypass: 0x{:05x} | ongoing: {} | jump_flag: {}",
             on_branch, br_sm[0], br_jump[0], should_fetch, ex_bypass[0], ongoing[0],jump_flag)
        icache.build(Bits(1)(0), real_fetch, to_fetch[2:2+depth_log-1].bitcast(Int(depth_log)), Bits(32)(0))
        log("on_br: {}         | de_by: {}     | fetch: {}      | addr: 0x{:05x} | new_cnt: {}",
            on_branch, ex_valid.optional(Bits(1)(0)), real_fetch, to_fetch, new_cnt)

        with Condition(real_fetch):
            decoder.async_called(fetch_addr=to_fetch)
            pc_reg[0] = (to_fetch.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
            ongoing[0] = new_cnt + Int(8)(1)
        
        with Condition(~real_fetch):
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
        
        reg_onwrite[0] = reg_onwrite[0] ^ ex_bit ^ wb_bit
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

        mem_bypass_reg = RegArray(bits5, 1)
        mem_bypass_data = RegArray(bits32, 1)

        wb_bypass_reg = RegArray(bits5, 1)
        wb_bypass_data = RegArray(bits32, 1)

        exec_br_dest = RegArray(Bits(32), 1)
        exec_br_jumped = RegArray(Bits(1), 1)
        mem_br_no_jump = RegArray(Bits(1), 1)
        d_br_buffer = RegArray(Bits(1), 1)
        icache = SRAM(width=32, depth=1<<depth_log, init_file=f"{workspace}/workload.exe")
        icache.name = 'icache'

        writeback = WriteBack()
        wb_rd = writeback.build(reg_file = reg_file)

        memory_access = MemoryAccess()

        driver = Driver()
        init_cache = driver.build(fetcher, user)

        executor = Execution()
        offset_reg = user.build(init_cache.dout)

        exec_rd, ex_valid, exec_br_jump,dcache = executor.build(
            pc = pc_reg,
            exec_bypass_reg = exec_bypass_reg,
            exec_bypass_data = exec_bypass_data,
            reg_onwrite = reg_onwrite,
            mem_bypass_reg = mem_bypass_reg,
            mem_bypass_data = mem_bypass_data,
            wb_bypass_reg = wb_bypass_reg,
            wb_bypass_data = wb_bypass_data,
            offset_reg = offset_reg,
            rf = reg_file,
            csr_f = csr_file,
            memory = memory_access,
            data = f'{workspace}/workload.data',
            #writeback = writeback,
            depth_log = depth_log,
            exec_br_dest = exec_br_dest,


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
        on_br = decoder.build(executor=executor,rdata=icache.dout)

        fetcher_impl.build(on_br, exec_br_dest, ex_valid, pc_reg,
                            pc_addr, decoder, 
                              depth_log, d_br_buffer , exec_br_jumped ,
                              mem_br_no_jump,exec_br_jump,icache)

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
        sim_threshold=600000,
        idle_threshold=600000,
        resource_base='',
        fifo_depth=1,
    )

    simulator_path, verilog_path = elaborate(sys, **conf)

    # Return the built system and relevant components
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

    report = False

    if report:
        raw = utils.run_simulator(simulator_path, False)
        open(f'{workload}.log', 'w').write(raw)
        #open(f'{workload}.sim.time', 'w').write(str(tt))
        raw = utils.run_verilator(verilog_path)
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
    args = py_sys.argv[1:]
    print("minor-CPU built successfully!")
    # Define workloads
    wl_path = f'{utils.repo_path()}/examples/minor-cpu/workloads'
    if not args:
        workloads = [
            #'0to100',
            #'median',
            #'multiply',
            #'qsort',
            #'rsort',
            #'towers',
            #'vvadd',
        ]
        # Iterate workloads
        for wl in workloads:
            # Copy workloads to tmp directory and rename to workload.
            init_workspace(wl_path, wl)
            run_cpu(sys, simulator_path, verilog_path , wl)
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
            #'rv32ui-p-lbu',#TO DEBUG&TO CHECK
            #'rv32ui-p-sb',#TO CHECK
        ]
        tests = f'{utils.repo_path()}/examples/minor-cpu/unit-tests'
        # Iterate test cases
        for case in test_cases:
            # Copy test cases to tmp directory and rename to workload.
            init_workspace(tests, case)
            run_cpu(sys, simulator_path, verilog_path)
        print("minor-CPU tests ran successfully!")
    else:
        # If user DID specify workloads, run exactly those, skipping default & tests:
        for wl in args:
            init_workspace(wl_path, wl)
            run_cpu(sys, simulator_path, verilog_path, wl)
        print("Done running user-specified workload(s)!")