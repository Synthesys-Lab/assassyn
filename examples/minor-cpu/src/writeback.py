from assassyn.frontend import *
from opcodes import *

class WriteBack(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'is_memory_read': Port(Bits(1)),
                'result': Port(Bits(32)),
                'rd': Port(Bits(5)),
                'mdata': Port(Bits(32)),
            }, no_arbiter=True)

        self.name = 'W'

    @module.combinational
    def build(self, reg_file: Array):

        is_memory_read, result, rd, mdata = self.pop_all_ports(True)

        data = is_memory_read.select(mdata, result)

        with Condition((rd != Bits(5)(0))):
            log("writeback        | x{:02}          | 0x{:08x}", rd, data)
            reg_file[rd] = data

        return rd
