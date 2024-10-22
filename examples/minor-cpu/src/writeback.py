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
                'is_csr': Port(Bits(1)),
                'csr_id': Port(Bits(4)),
                'csr_new': Port(Bits(32))
            }, no_arbiter=True)

        self.name = 'W'

    @module.combinational
    def build(self, reg_file: Array , csr_file: Array):

        is_memory_read, result, rd, mdata , is_csr , csr_id , csr_new= self.pop_all_ports(True)

        data = is_memory_read.select(mdata, result)

        with Condition((rd != Bits(5)(0))):
            log("writeback        | x{:02}          | 0x{:08x}", rd, data)
            reg_file[rd] = data

        with Condition(is_csr):
            log("writeback        | csr[{:02}]       | 0x{:08x}", csr_id, data)
            csr_file[csr_id] = csr_new

        return rd
