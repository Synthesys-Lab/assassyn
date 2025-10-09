 
from assassyn.frontend import *
from opcodes import *
class MemoryAccess(Module):
    
    def __init__(self):
        super().__init__(
            ports={ 'rd': Port(Bits(5)),'mem_ext' : Port(Bits(2)),'result': Port(Bits(32)),'is_mem_read': Port(Bits(1))},
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

        data_cut = Bits(32)(0)
        mem_ext = self.mem_ext.pop()
        result = self.result.pop()
        rd = self.rd.pop()
        is_mem_read = self.is_mem_read.pop()
        data = rdata[0].bitcast(Bits(32))
        with Condition(is_mem_read):
            log("mem.rdata        | 0x{:x}", data)
            mem_bypass_reg[0] = rd
            with Condition(rd != Bits(5)(0)):
                log("mem.bypass       | x{:02} = 0x{:x}", rd, data)
                mem_bypass_data[0] = data

        with Condition(~is_mem_read):
            mem_bypass_reg[0] = Bits(5)(0)

        arg = is_mem_read.select(data, Bits(32)(0))
        sign = arg[7:7]
        ext = sign.select(Bits(24)(0xffffff), Bits(24)(0))
        data_cut = mem_ext[1:1].select( Bits(24)(0).concat(arg[0:7]) , ext.concat(arg[0:7]) )
        
        arg = is_mem_read.select(arg, result)
        arg = mem_ext[0:0].select( data_cut ,arg)

        wb_bypass_data[0] = arg
        wb_bypass_reg[0] = rd

        wb_bound = writeback.bind(mdata = arg , rd = rd)
        wb_bound.async_called() 
