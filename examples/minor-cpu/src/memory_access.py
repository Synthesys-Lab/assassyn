 
from assassyn.frontend import *
from opcodes import *
class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={
                'rd': Port(Bits(5)),
                'mem_size': Port(Bits(2)),
                'mem_unsigned': Port(Bits(1)),
                'addr_lsb': Port(Bits(2)),
                'result': Port(Bits(32)),
                'is_mem_read': Port(Bits(1)),
            },
            no_arbiter=True)
        self.name = 'm'

    @module.combinational
    def build(
        self, 
        writeback: Module, 
        mem_bypass_reg: Array, 
        mem_bypass_data: Array,
        wb_bypass_data: Array,
        wb_bypass_reg: Array,
        rdata:RegArray
    ):
        self.timing = 'systolic'

        mem_size = self.mem_size.pop()
        mem_unsigned = self.mem_unsigned.pop()
        addr_lsb = self.addr_lsb.pop()
        result = self.result.pop()
        rd = self.rd.pop()
        is_mem_read = self.is_mem_read.pop()
        data = rdata[0].bitcast(Bits(32))

        is_half = mem_size == Bits(2)(1)
        is_byte = mem_size == Bits(2)(2)

        byte_data = data[0:7]
        byte_data = (addr_lsb == Bits(2)(1)).select(data[8:15], byte_data)
        byte_data = (addr_lsb == Bits(2)(2)).select(data[16:23], byte_data)
        byte_data = (addr_lsb == Bits(2)(3)).select(data[24:31], byte_data)

        half_data = addr_lsb[1:1].select(data[16:31], data[0:15])

        byte_sign = byte_data[7:7].select(Bits(24)(0xffffff), Bits(24)(0))
        half_sign = half_data[15:15].select(Bits(16)(0xffff), Bits(16)(0))

        byte_value = mem_unsigned.select(Bits(24)(0).concat(byte_data), byte_sign.concat(byte_data))
        half_value = mem_unsigned.select(Bits(16)(0).concat(half_data), half_sign.concat(half_data))

        load_value = is_byte.select(byte_value, is_half.select(half_value, data))
        arg = is_mem_read.select(load_value, result)

        with Condition(is_mem_read):
            log("mem.rdata        | 0x{:x}", data)
            log("mem.loaded       | x{:02} = 0x{:08x}", rd, load_value)
            mem_bypass_reg[0] = rd
            mem_bypass_data[0] = load_value

        with Condition(~is_mem_read):
            with Condition(rd != Bits(5)(0)):
                log("mem.pass         | x{:02} = 0x{:08x}", rd, result)
            mem_bypass_reg[0] = Bits(5)(0)
            mem_bypass_data[0] = Bits(32)(0)

        wb_bypass_data[0] = arg
        wb_bypass_reg[0] = rd

        wb_bound = writeback.bind(mdata = arg , rd = rd)
        wb_call = wb_bound.async_called()
        wb_call.bind.set_fifo_depth(mdata=2, rd=2)
