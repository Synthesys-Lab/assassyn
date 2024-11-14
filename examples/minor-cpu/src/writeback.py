from assassyn.frontend import *
from opcodes import *

class WriteBack(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'rd': Port(Bits(5)),
                'mdata': Port(Bits(32)),
            }, no_arbiter=True)

        self.name = 'W'

    @module.combinational
    def build(self, reg_file: Array ):

        rd, mdata = self.pop_all_ports(False)
        with Condition((rd != Bits(5)(0))):
            log("writeback        | x{:02}          | 0x{:08x}", rd, mdata)
            reg_file[rd] = mdata
        return rd
