from assassyn.frontend import *
from opcodes import *

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
