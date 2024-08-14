import pytest

from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils

class Execution(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.opcode = Port(Bits(7))
        self.imm_value = Port(Bits(32))
        self.a_reg = Port(Bits(5))
        self.b_reg = Port(Bits(5))
        self.rd_reg = Port(Bits(5))

    @module.combinational
    def build(self):
        log("executing")

    @module.wait_until
    def wait_until(self):
        # Handle read after write
        pass

class Decoder(Memory):
    
    @module.constructor
    def __init__(self, init_file):
        super().__init__(width=32, depth=1024, latency=(1, 1), init_file=init_file)

    @module.combinational
    def build(self, inst, pc, on_branch, exec):
        with Condition(~on_branch[0]):
            # Slice the fields
            opcode = inst[0:6]
            log("decoding: {:b}", opcode)
            rd = inst[7:11]
            rs1 = inst[15:19]
            rs2 = inst[20:24]
            i_imm = inst[20:31]
            
            u_imm = inst[12:31].concat(Bits(12)(0)) 

            sign = inst[31:31]
            b_imm = inst[31:31].concat(inst[7:7]).concat(inst[25:30]).concat(inst[8:11]).concat(Bits(1)(0))
            b_imm = sign.select(Bits(19)(0x7ffff), Bits((19)(0))).concat(b_imm)

            is_lui  = opcode == Bits(6)(0b0110111)
            is_addi = opcode == Bits(6)(0b0010011)
            is_add  = opcode == Bits(6)(0b0110011)
            is_lw   = opcode == Bits(6)(0b0000011)
            is_bne  = opcode == Bits(6)(0b1100011)
            is_ret  = opcode == Bits(6)(0b1101111)

            supported = is_lui | is_addi | is_add | is_lw | is_bne | is_ret
            write_rd = is_lui | is_addi | is_add | is_lw
            read_rs1 = is_lui | is_addi | is_add | is_bne | is_lw
            read_rs2 = is_add | is_bne
            read_i_imm = is_addi | is_lw
            read_u_imm = is_lui
            read_b_imm = is_bne
            
            with Condition(is_bne):
                log("set on-branch!")
                on_branch[0] = Bits(1)(1)

            reg_a = read_rs1.select(rs1, Bits(5)(0))
            reg_b = read_rs2.select(rs2, Bits(5)(0))

            no_imm = ~(read_i_imm | read_u_imm | read_b_imm)
            imm_cond = read_i_imm.concat(read_u_imm).concat(read_b_imm).concat(no_imm)
            # {read_i_imm, read_u_imm, read_b_imm, no_imm}
            imm_value = imm_cond.select1hot(Bits(32)(0), b_imm, u_imm, i_imm.zext(Bits(32)))
            
            rd_reg = write_rd.select(rd, Bits(5)(0))

            #TODO: parameter list
            exec.async_called()

            with Condition(is_lui):
                log("lui:   rd: x{}, imm: 0x{:x}", rd, imm_value)
            with Condition(is_lw):
                log("lw:    rd: x{}, rs1: x{}, imm: {}", rd, rs1, i_imm)
            with Condition(is_addi):
                log("addi:  rd: x{}, rs1: x{}, imm: {}", rd, rs1, i_imm)
            with Condition(is_add):
                log("add:   rd: x{}, rs1: x{}, rs2: x{}", rd, rs1, rs2)
            with Condition(is_bne):
                log("bne:   rs1:x{}, rs2: x{}, imm: {}", rs1, rs2, b_imm.bitcast(Int(32)))
            with Condition(is_ret):
                log("ret")
            
            with Condition(~supported):
                log("unsupported opcode: {:b}, raw_inst: {:x}", opcode, inst)
        
        with Condition(on_branch[0]):
            log("on a branch, stall decoding, pc freeze at 0x{:x}", pc[0])
                
    @module.wait_until
    def wait_until(self):
        return self.validate_all_ports()

class Fetcher(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, decoder: Memory, pc: Array, on_branch: Array):
        with Condition(~on_branch[0]):
            log("Fetching instruction at PC: 0x{:x}", pc[0])
            to_fetch = pc[0][2:11].bitcast(UInt(10))
            decoder.async_called(write = Bits(1)(0), wdata = Bits(32)(0), addr = to_fetch)
            pc[0] = (pc[0].bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))

        with Condition(on_branch[0]):
            log("on a branch, stall fetching, pc freeze at 0x{:x}", pc[0])


class Driver(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, fetcher: Module):
        fetcher.async_called()