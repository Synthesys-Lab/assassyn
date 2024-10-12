''' A simplest single issue RISCV CPU, which has no operand buffer.
'''

import pytest

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

from opcodes import *

class InstType:

    FIELDS = [
        ((0, 6), 'opcode', Bits),
        ((7, 11), 'rd', Bits),
        ((15, 19), 'rs1', Bits),
        ((20, 24), 'rs2', Bits),
        ((12, 14), 'funct3', Bits),
        ((25, 31), 'funct7', Bits),
    ]

    def __init__(self, rd, rs1, rs2, funct3, funct7, fields):
        self.fields = fields.copy()
        for cond, entry in zip([True, rd, rs1, rs2, funct3, funct7], self.FIELDS):
            key, field, ty = entry
            setattr(self, f'has_{field}', cond)
            if cond:
                self.fields[key] = (field, ty)
        self.dtype = Record(self.fields)
        self.value = None

    def view(self):
        return self.dtype.view(self.value)

class RInst(InstType):

    PREFIX = 'r'

    def __init__(self, value):
        super().__init__(True, True, True, True, True, {})
        self.value = value

    def imm(self, pad):
        return None

class IInst(InstType):

    PREFIX = 'i'

    def __init__(self, value):
        super().__init__(True, True, False, True, False, { (20, 31): ('imm', Bits) })
        self.value = value

    def imm(self, pad):
        raw = self.view().imm
        if pad:
            raw = concat(Bits(20)(0), raw)
        return raw

class SInst(InstType):

    PREFIX = 's'

    def __init__(self, value):
        fields = { (25, 31): ('imm11_5', Bits), (7, 11): ('imm4_0', Bits) }
        super().__init__(False, True, True, True, False, fields)
        self.value = value

    def imm(self, pad):
        imm = self.view().imm11_5.concat(self.view().imm4_0)
        if pad:
            imm = concat(Bits(20)(0), imm)
        return imm

class UInst(InstType):

    PREFIX = 'u'

    def __init__(self, value):
        super().__init__(True, False, False, False, False, { (12, 31): ('imm', Bits) })
        self.value = value

    def imm(self, pad):
        raw = self.view().imm
        if pad:
            raw = concat(Bits(12)(0), raw)
        return raw

class BInst(InstType):

    PREFIX = 'b'

    def __init__(self, value):
        fields = {
            (7, 7): ('imm11', Bits),
            (8, 11): ('imm4_1', Bits),
            (25, 30): ('imm10_5', Bits),
            (31, 31): ('imm12', Bits),
        }
        super().__init__(False, True, True, True, False, fields)
        self.value = value

    def imm(self, pad):
        imm = concat(self.view().imm12, self.view().imm11, self.view().imm10_5, self.view().imm4_1)
        imm = imm.concat(Bits(1)(0))
        if pad:
            imm = concat(Bits(19)(0), imm)
        return imm

supported_opcodes = [
  # mn,     opcode,    type
  ('lui'   , 0b0110111, UInst),
  ('addi'  , 0b0010011, IInst),
  ('add'   , 0b0110011, RInst),
  ('lw'    , 0b0000011, IInst),
  ('bne'   , 0b1100011, BInst),
  ('ret'   , 0b1101111, UInst),
  ('ebreak', 0b1110011, IInst),
]

deocder_signals = Record(
  memory_read=Bits(1),
  invoke_adder=Bits(1),
  is_branch=Bits(1),
  rs1_reg=Bits(5),
  rs1_valid=Bits(1),
  rs2_reg=Bits(5),
  rs2_valid=Bits(1),
  rd_reg=Bits(5),
  rd_valid=Bits(1),
  imm_valid=Bits(1),
  imm_value=Bits(32),
)

supported_types = [RInst, IInst, SInst, BInst, UInst]

def decode_logic(inst):

    views = {i: i(inst) for i in supported_types}
    is_type = {i: Bits(1)(0) for i in supported_types}

    eqs = {}

    rd_valid = Bits(1)(0)
    rs1_valid = Bits(1)(0)
    rs2_valid = Bits(1)(0)
    imm_valid = Bits(1)(0)

    # Check if the given instruction's opcode equals one of the supported opcodes
    for mn, opcode, cur_type in supported_opcodes:

        ri = views[cur_type]
        wrapped_opcode = Bits(7)(opcode)
        eq = ri.view().opcode == wrapped_opcode
        is_type[cur_type] = is_type[cur_type] | eq
        eqs[mn] = eq

        pad = 6 - len(mn)
        pad = ' ' * pad


        fmt = None
        str_opcode = bin(opcode)[2:]
        str_opcode = (7 - len(str_opcode)) * '0' + str_opcode
        fmt = f"{cur_type.PREFIX}.{mn}.{str_opcode}{pad} "

        args = []

        if ri.has_rd:
            fmt = fmt + "| rd: x{:02}      "
            args.append(ri.view().rd)
            rd_valid = rd_valid | eq
        else:
            fmt = fmt + '|               '

        if ri.has_rs1:
            fmt = fmt + "| rs1: x{:02}      "
            args.append(ri.view().rs1)
            rs1_valid = rs1_valid | eq
        else:
            fmt = fmt + '|               '

        if ri.has_rs2:
            fmt = fmt + "| rs2: x{:02}      "
            args.append(ri.view().rs2)
            rs2_valid = rs2_valid | eq
        else:
            fmt = fmt + '|               '

        imm = ri.imm(False)
        if imm is not None:
            fmt = fmt + "|imm: 0x{:x}"
            args.append(imm)


        with Condition(eq):
            log(fmt, *args)


    # Extract all the signals
    memory_read = eqs['lw']
    invoke_adder = eqs['addi'] | eqs['add'] | eqs['lw'] | eqs['bne']
    is_branch = eqs['bne'] | eqs['ret'] | eqs['ebreak']
    # Extract all the operands according to the instruction types
    # rd
    rd_reg = rd_valid.select(views[RInst].view().rd, Bits(5)(0))
    # rs1
    rs1_reg = rs1_valid.select(views[RInst].view().rs1, Bits(5)(0))
    # rs2
    rs2_reg = rs2_valid.select(views[RInst].view().rs2, Bits(5)(0))
    # imm
    imm_valid = is_type[IInst] | is_type[UInst] | is_type[SInst] | is_type[BInst]
    imm_value = Bits(32)(0)
    for i in supported_types:
        new_imm = views[i].imm(True)
        if new_imm is not None:
            imm_value = is_type[i].select(new_imm, imm_value)
    imm_value = eqs['lui'].select(views[UInst].imm(False).concat(Bits(12)(0)), imm_value)

    return deocder_signals.bundle(
        memory_read=memory_read,
        invoke_adder=invoke_adder,
        is_branch=is_branch,
        rs1_reg=rs1_reg,
        rs1_valid=rs1_valid,
        rs2_reg=rs2_reg,
        rs2_valid=rs2_valid,
        rd_reg=rd_reg,
        rd_valid=rd_valid,
        imm_valid=imm_valid,
        imm_value=imm_value)

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

class WriteBack(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'opcode': Port(Bits(7)),
                'result': Port(Bits(32)),
                'rd': Port(Bits(5)),
                'mdata': Port(Bits(32)),
            }, no_arbiter=True)

        self.name = 'WriteBack'

    @module.combinational
    def build(self, reg_file: Array):

        opcode, result, rd, mdata = self.pop_all_ports(True)

        op_check = OpcodeChecker(opcode)
        op_check.check('lui', 'addi', 'add', 'lw', 'bne', 'ret')

        is_lui  = op_check.lui
        is_addi = op_check.addi
        is_add  = op_check.add
        is_lw   = op_check.lw
        is_bne  = op_check.bne
        # is_ret  = op_check.ret

        is_result = is_lui | is_addi | is_add | is_bne
        is_memory = is_lw
        cond = is_memory.concat(is_result)
        # {is_memory, is_result}
        data = cond.select1hot(result, mdata)

        return_rd = None

        with Condition((rd != Bits(5)(0))):
            log("writeback        | x{:02} = 0x{:x}", rd, data)
            reg_file[rd] = data
            return_rd = rd

        return return_rd

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
 

class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={'rdata': Port(Bits(32))},
            no_arbiter=True)
        self.name = 'memaccess'

    @module.combinational
    def build(
        self, 
        writeback: Module, 
        mem_bypass_reg: Array, 
        mem_bypass_data: Array
    ):
        self.timing = 'systolic'

        with Condition(self.rdata.valid()):
            data = self.rdata.pop()
            log("mem.rdata        | 0x{:x}", data)
            with Condition(mem_bypass_reg[0] != Bits(5)(0)):
                log("mem.bypass       | x{:02} = 0x{:x}", mem_bypass_reg[0], data)
            mem_bypass_data[0] = (mem_bypass_reg[0] != Bits(5)(0)).select(data, Bits(32)(0))

        arg = self.rdata.valid().select(self.rdata.peek(), Bits(32)(0))
        writeback.async_called(mdata = arg)

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

def check(raw):
    data_path = f'{utils.repo_path()}/examples/minor-cpu/resource/0to100.data'
    with open(data_path, 'r') as f:
        data = []
        for line in f:
            line = line.split('//')[0].strip()
            if line and not line.startswith('@'):
                try:
                    data.append(int(line, 16))
                except ValueError:
                    print(f"Warning: Skipping invalid line: {line}")


    accumulator = 0
    ideal_accumulator = 0
    data_index = 0

    for line in raw.split('\n'):

        if 'writeback' in line and 'x14 = ' in line:
            loaded = int(line.split('=')[-1].strip(), 16)
            assert data[data_index] == loaded, f"Data mismatch at step {data_index + 1}: {hex(data[data_index])} != {hex(loaded)}"

        if 'writeback' in line and 'x15 = ' in line:
            addr = int(line.split('=')[-1].strip(), 16)
            if addr == 0 or addr == 0xb8:
                continue
            assert 0xb8 + (data_index + 1) * 4 == addr, f"Address mismatch at step {data_index + 1}: {hex(addr)} != {hex(0xb8 + (data_index + 1) * 4)}"

        if 'writeback' in line and 'x10 = ' in line:
            value = int(line.split('=')[-1].strip(), 16)
            if value != accumulator:
                accumulator = value
                if data_index < len(data):
                    ideal_accumulator += data[data_index]
                    assert accumulator == ideal_accumulator, f"Mismatch at step {data_index + 1}: CPU {accumulator} != Reference {ideal_accumulator}"
                    data_index += 1

    assert data_index == 100, f"Data index mismatch: {data_index} != 100"

    print(f"Final CPU sum: {accumulator} (0x{accumulator:x})")
    print(f"Final ideal sum: {ideal_accumulator} (0x{ideal_accumulator:x})")
    print(f"Final difference: {accumulator - ideal_accumulator}")

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
