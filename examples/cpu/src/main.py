import pytest

from assassyn.frontend import *
from assassyn.backend import *
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
    def build(
        self, 
        reg_onwrite: Array, 
        on_branch: Array, 
        pc: Array, 
        exec_bypass_reg: Array, 
        exec_bypass_data: Array, 
        mem_bypass_reg: Array, 
        mem_bypass_data: Array, 
        rf: Array, 
        memory: Memory, 
        writeback: Module
    ):
        log("executing: {:b}", self.opcode)

        with Condition(self.rd_reg != Bits(5)(0)):
            reg_onwrite[self.rd_reg] = Bits(1)(1)
            log("set x{} as on-write", self.rd_reg)

        is_lui  = self.opcode == Bits(7)(0b0110111)
        is_addi = self.opcode == Bits(7)(0b0010011)
        is_add  = self.opcode == Bits(7)(0b0110011)
        is_lw   = self.opcode == Bits(7)(0b0000011)
        is_bne  = self.opcode == Bits(7)(0b1100011)
        # is_ret  = self.opcode == Bits(7)(0b1101111)

        # Instruction attributes
        uses_imm = is_addi | is_bne
        is_branch = is_bne

        a = (exec_bypass_reg[0] == self.a_reg).select(
            exec_bypass_data[0], 
            (mem_bypass_reg[0] == self.a_reg).select(mem_bypass_data[0], rf[self.a_reg])
        )
        b = (exec_bypass_reg[0] == self.b_reg).select(
            exec_bypass_data[0], 
            (mem_bypass_reg[0] == self.b_reg).select(mem_bypass_data[0], rf[self.b_reg])
        )
        
        rhs = uses_imm.select(self.imm_value, b)

        invokde_adder = is_add | is_addi | is_lw

        result = (a.bitcast(Int(32)) + rhs.bitcast(Int(32))).bitcast(Bits(32))
        result = (invokde_adder.concat(is_lui).concat(is_branch)).select1hot(
        # {invokde_adder, is_lui, is_branch}
            Bits(32)(0), self.imm_value, result
        )
        log("{:b}: a: {:x}, b:{:x}, res: {:x}", self.opcode, a, rhs, result)

        produced_by_exec = is_lui | is_addi | is_add

        exec_bypass_reg[0] = produced_by_exec.select(self.rd_reg, Bits(5)(0))
        exec_bypass_data[0] = produced_by_exec.select(result, Bits(32)(0))

        with Condition(is_branch):
            on_branch[0] = Bits(1)(0)
            log("reset on-branch")
        
        with Condition(is_bne):
            new_pc = (pc[0].bitcast(Int(32)) - Int(32)(8) \
                      + self.imm_value.bitcast(Int(32))).bitcast(Bits(32))
            br_dest = (a != b).select(new_pc, pc[0])
            log("if {} != {}: branch to {}; actual: {}", a, b, new_pc, br_dest)

        is_memory = is_lw
        is_memory_read = is_lw

        request_addr = is_memory.select(result[2:19].bitcast(Int(17)), Int(17)(0))

        mem_bypass_reg[0] = is_memory_read.select(self.rd_reg, Bits(5)(0))

        with Condition(is_memory):
            log("addr: {:x}, lineno: {:x}", result, request_addr)

        memory.async_called(we = Int(1)(0), wdata = a, addr = request_addr)
        wb = writeback.bind(opcode = self.opcode, result = result, rd = self.rd_reg)

        return wb


    @module.wait_until
    def wait_until(self, exec_bypass_reg: Array, mem_bypass_reg: Array, reg_onwrite: Array):
        # Handle read after write
        a_valid = (~reg_onwrite[self.a_reg.peek()])      | \
            (exec_bypass_reg[0] == self.a_reg.peek())    | \
            (mem_bypass_reg[0] == self.a_reg.peek())
        b_valid = (~reg_onwrite[self.b_reg.peek()])      | \
            (exec_bypass_reg[0] == self.b_reg.peek())    | \
            (mem_bypass_reg[0] == self.b_reg.peek())

        rd_valid = ~reg_onwrite[self.rd_reg.peek()]

        valid = a_valid & b_valid & rd_valid
        with Condition(~valid):
            log("operand not ready, stall execution, x{}: {}, x{}: {}, x{}: {}", \
                self.a_reg.peek(), a_valid, \
                self.b_reg.peek(), b_valid, \
                self.rd_reg.peek(), rd_valid)
        return valid

    
class WriteBack(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.opcode = Port(Bits(7))
        self.result = Port(Bits(32))
        self.rd     = Port(Bits(5)) 
        self.mdata  = Port(Bits(32))

    @module.combinational
    def build(self, reg_file: Array, reg_onwrite: Array):
        is_lui  = self.opcode == Bits(7)(0b0110111)
        is_addi = self.opcode == Bits(7)(0b0010011)
        is_add  = self.opcode == Bits(7)(0b0110011)
        is_lw   = self.opcode == Bits(7)(0b0000011)
        is_bne  = self.opcode == Bits(7)(0b1100011)
        # is_ret  = self.opcode == Bits(7)(0b1101111)

        is_result = is_lui | is_addi | is_add | is_bne
        is_memory = is_lw
        cond = is_memory.concat(is_result)
        # {is_memory, is_result}
        data = cond.select1hot(self.result, self.mdata)

        with Condition((self.rd != Bits(5)(0))):
            log("opcode: {:b}, writeback: x{} = {:x}", self.opcode, self.rd, data)
            reg_file[self.rd] = data
            reg_onwrite[self.rd] = Bits(1)(0)

class Decoder(Memory):
    
    @module.constructor
    def __init__(self, init_file):
        super().__init__(width=32, depth=1024, latency=(1, 1), init_file=init_file)

    @module.combinational
    def build(self, pc: Array, on_branch: Array, exec: Module):
        super().build()
        inst = self.rdata
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
            b_imm = inst[31:31].concat(inst[7:7]) \
                               .concat(inst[25:30]) \
                               .concat(inst[8:11]) \
                               .concat(Bits(1)(0))
            b_imm = (sign.select(Bits(19)(0x7ffff), Bits(19)(0))).concat(b_imm)

            is_lui  = opcode == Bits(7)(0b0110111)
            is_addi = opcode == Bits(7)(0b0010011)
            is_add  = opcode == Bits(7)(0b0110011)
            is_lw   = opcode == Bits(7)(0b0000011)
            is_bne  = opcode == Bits(7)(0b1100011)
            is_ret  = opcode == Bits(7)(0b1101111)

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
            imm_cond = read_i_imm.concat(read_u_imm) \
                                 .concat(read_b_imm) \
                                 .concat(no_imm)
            # {read_i_imm, read_u_imm, read_b_imm, no_imm}
            imm_value = imm_cond.select1hot(
                Bits(32)(0), 
                b_imm, 
                u_imm, 
                i_imm.zext(Bits(32))
            )
            
            rd_reg = write_rd.select(rd, Bits(5)(0))

            exec.async_called(
                opcode = opcode, 
                imm_value = imm_value, 
                a_reg = reg_a, 
                b_reg = reg_b, 
                rd_reg = rd_reg
            )

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

class MemoryAccess(Memory):
    
    @module.constructor
    def __init__(self, init_file):
        super().__init__(width=32, depth=65536 * 2, latency=(1, 1), init_file=init_file)

    @module.combinational
    def build(self, writeback: Module, mem_bypass_reg: Array, mem_bypass_data: Array):
        super().build()
        data = self.rdata
        log("mem-data: 0x{:x}", data)
        writeback.async_called(mdata = data)
        with Condition(mem_bypass_reg[0] != Bits(5)(0)):
            log("bypass memory data: x{} = {}", mem_bypass_reg[0], data)
        mem_bypass_data[0] = (mem_bypass_reg[0] != Bits(5)(0)).select(data, Bits(32)(0))

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
            to_fetch = pc[0][2:11].bitcast(Int(10))
            decoder.async_called(we = Int(1)(0), wdata = Bits(32)(0), addr = to_fetch)
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


def main():
    sys = SysBuilder('cpu')

    with sys:
        # Data Types
        bits1   = Bits(1)
        bits5   = Bits(5)
        bits32  = Bits(32)

        # Data Structures
        pc          = RegArray(bits32, 1)
        on_branch   = RegArray(bits1, 1)
        reg_file    = RegArray(bits32, 32)
        reg_onwrite = RegArray(bits1, 32, attr=[Array.FULLY_PARTITIONED]) #TODO: Make it downstream

        exec_bypass_reg = RegArray(bits5, 1)
        exec_bypass_data = RegArray(bits32, 1)

        mem_bypass_reg = RegArray(bits5, 1)
        mem_bypass_data = RegArray(bits32, 1)


        writeback = WriteBack()
        writeback.build(reg_file, reg_onwrite)

        memory_access = MemoryAccess('0to100.data')
        memory_access.wait_until()
        memory_access.build(writeback = writeback, mem_bypass_reg = mem_bypass_reg, mem_bypass_data=mem_bypass_data)

        exec = Execution()
        exec.wait_until(exec_bypass_reg, mem_bypass_reg, reg_onwrite)
        wb = exec.build(
            pc = pc,
            on_branch=on_branch,
            reg_onwrite=reg_onwrite,
            exec_bypass_reg = exec_bypass_reg,
            exec_bypass_data = exec_bypass_data,
            mem_bypass_reg = mem_bypass_reg,
            mem_bypass_data = mem_bypass_data,
            rf = reg_file,
            memory = memory_access,
            writeback = writeback
        )

        decoder = Decoder('0to100.data')
        decoder.wait_until()
        decoder.build(pc = pc, on_branch = on_branch, exec = exec)
    
        fetcher = Fetcher()
        fetcher.build(decoder, pc, on_branch)

        driver = Driver()
        driver.build(fetcher)

    print(sys)
    conf = config(
        verilog=None,
        sim_threshold=100,
        idle_threshold=100,
        resource_base=f'{utils.repo_path()}/examples/cpu/resource'
    )

    simulator_path, _ = elaborate(sys, **conf)

    raw = utils.run_simulator(simulator_path)
    print(raw)

if __name__ == '__main__':
    main()