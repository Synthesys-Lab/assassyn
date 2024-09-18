import pytest
import re
from assassyn import backend
from assassyn.dtype import *
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.expr import Bind
from systolic_array_rev import ProcElem, Sink, ColPusher, RowPusher, ComputePE, check_raw

#  # PE Array (4 + 1) x (4 + 1)
#           [Pusher]      [Pusher]      [Pusher]      [Pusher]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#           [Sink]        [Sink]        [Sink]        [Sink]

class SRAM_R(Memory):

    @module.constructor
    def __init__(self, init_file, width):
        super().__init__(width=width, depth=1024, latency=(1, 1), init_file=init_file)

    @module.combinational
    def build(self, width, \
              row1: RowPusher, row2: RowPusher, row3: RowPusher, row4: RowPusher
              ):
        super().build()
        read = ~self.we
        cnt = RegArray(Int(width), 1)
        buffer = RegArray(Int(width), 4)
        with Condition(read):
            rdata = self.rdata.bitcast(Int(128))
            
            with Cycle(2):
                buffer[0] = rdata
                
            with Cycle(3):
                buffer[1] = rdata
                row1.async_called(data = buffer[0][96:127].bitcast(Int(32)))
                log("Row Buffer data (hex): {:#034x}", buffer[0]) 
            
            with Cycle(4):
                buffer[2] = rdata
                row1.async_called(data = buffer[0][64:95].bitcast(Int(32)))
                row2.async_called(data = buffer[1][96:127].bitcast(Int(32)))
                log("Row Buffer data (hex): {:#034x}", buffer[1]) 
            
            with Cycle(5):
                buffer[3] = rdata
                row1.async_called(data = buffer[0][32:63].bitcast(Int(32)))
                row2.async_called(data = buffer[1][64:95].bitcast(Int(32)))
                row3.async_called(data = buffer[2][96:127].bitcast(Int(32)))
                log("Row Buffer data (hex): {:#034x}", buffer[2]) 
            
            with Cycle(6):
                log("Row Buffer data (hex): {:#034x}", buffer[3])
                row1.async_called(data = buffer[0][0:31].bitcast(Int(32)))
                row2.async_called(data = buffer[1][32:63].bitcast(Int(32)))
                row3.async_called(data = buffer[2][64:95].bitcast(Int(32)))
                row4.async_called(data = buffer[3][96:127].bitcast(Int(32)))

            with Cycle(7):
                row2.async_called(data = buffer[1][0:31].bitcast(Int(32)))
                row3.async_called(data = buffer[2][32:63].bitcast(Int(32)))
                row4.async_called(data = buffer[3][64:95].bitcast(Int(32)))

            with Cycle(8):
                row3.async_called(data = buffer[2][0:31].bitcast(Int(32)))
                row4.async_called(data = buffer[3][32:63].bitcast(Int(32)))

            with Cycle(9):
                row4.async_called(data = buffer[3][0:31].bitcast(Int(32)))
        
        cnt[0] = cnt[0] + Int(width)(1)

    @module.wait_until
    def wait_until(self):
        return self.validate_all_ports()
    
class SRAM_C(Memory):

    @module.constructor
    def __init__(self, init_file, width):
        super().__init__(width=width, depth=1024, latency=(1, 1), init_file=init_file)

    @module.combinational
    def build(self, width, \
              col1: ColPusher, col2: ColPusher, col3: ColPusher, col4: ColPusher
              ):
        super().build()
        read = ~self.we
        cnt = RegArray(Int(width), 1)
        buffer = RegArray(Int(width), 4)
        with Condition(read):
            rdata = self.rdata.bitcast(Int(128))
            
            with Cycle(2):
                buffer[0] = rdata
                
            with Cycle(3):
                buffer[1] = rdata
                col1.async_called(data = buffer[0][96:127].bitcast(Int(32)))
                log("Col Buffer data (hex): {:#034x}", buffer[0]) 
            
            with Cycle(4):
                buffer[2] = rdata
                col1.async_called(data = buffer[0][64:95].bitcast(Int(32)))
                col2.async_called(data = buffer[1][96:127].bitcast(Int(32)))
                log("Col Buffer data (hex): {:#034x}", buffer[1]) 
            
            with Cycle(5):
                buffer[3] = rdata
                col1.async_called(data = buffer[0][32:63].bitcast(Int(32)))
                col2.async_called(data = buffer[1][64:95].bitcast(Int(32)))
                col3.async_called(data = buffer[2][96:127].bitcast(Int(32)))
                log("Col Buffer data (hex): {:#034x}", buffer[2]) 
            
            with Cycle(6):
                log("Col Buffer data (hex): {:#034x}", buffer[3])
                col1.async_called(data = buffer[0][0:31].bitcast(Int(32)))
                col2.async_called(data = buffer[1][32:63].bitcast(Int(32)))
                col3.async_called(data = buffer[2][64:95].bitcast(Int(32)))
                col4.async_called(data = buffer[3][96:127].bitcast(Int(32)))

            with Cycle(7):
                col2.async_called(data = buffer[1][0:31].bitcast(Int(32)))
                col3.async_called(data = buffer[2][32:63].bitcast(Int(32)))
                col4.async_called(data = buffer[3][64:95].bitcast(Int(32)))

            with Cycle(8):
                col3.async_called(data = buffer[2][0:31].bitcast(Int(32)))
                col4.async_called(data = buffer[3][32:63].bitcast(Int(32)))

            with Cycle(9):
                col4.async_called(data = buffer[3][0:31].bitcast(Int(32)))

        cnt[0] = cnt[0] + Int(width)(1)

    @module.wait_until
    def wait_until(self):
        return self.validate_all_ports()

class Driver(Module):

    @module.constructor
    def __init__(self):
        super().__init__()

    @module.combinational
    def build(self, memory_R: SRAM_R, memory_C: SRAM_C):
        cnt = RegArray(Int(memory_R.width), 1)
        v = cnt[0]
        we = Int(1)(0)
        raddr = v[0:9]  
        addr = raddr.bitcast(Int(10)) 
        cond = cnt[0] >= Int(128)(4)

        memory_R.async_called(
            we = we, 
            addr = addr,
            wdata = Bits(memory_R.width)(0)
            )  
            
        memory_C.async_called(
            we = we, 
            addr = addr,
            wdata = Bits(memory_C.width)(0)
            )  
        
        with Condition(cond):
            log('Asyn Call to Empty Memory')

        cnt[0] = cnt[0] + Int(memory_R.width)(1)

        
def mem_systolic_array(sys_name, width, init_file_row, init_file_col, resource_base):
    sys = SysBuilder(sys_name)
    pe_array = [[ProcElem() for _ in range(6)] for _ in range(6)]

    with sys:

        # Init ComputePE
        for i in range(1, 5):
            for j in range(1, 5):
                pe_array[i][j].pe = ComputePE()

        for i in range(1, 5):
            for j in range(1, 5):
                pe_array[i][j].bound = pe_array[i][j].pe

        # First Column Pushers
        for i in range(1, 5):
            pe_array[i][0].pe = ColPusher()
            if pe_array[i][1].bound is not None:  
                bound = pe_array[i][0].pe.build(pe_array[i][1].bound)
                pe_array[i][0].bound = bound
            else:
                print(f"Error: pe_array[{i}][1].bound is not initialized!")

        # First Row Pushers
        for i in range(1, 5):
            pe_array[0][i].pe = RowPusher()
            if pe_array[1][i].bound is not None:
                bound = pe_array[0][i].pe.build(pe_array[1][i].bound)
                pe_array[0][i].bound = bound
            else:
                print(f"Error: pe_array[1][{i}].bound is not initialized!")

        # Last Column Sink
        for i in range(1, 5):
            pe_array[i][5].pe = Sink('west')
            pe_array[i][5].pe.build()
            pe_array[i][5].bound = pe_array[i][5].pe

        # Last Row Sink
        for i in range(1, 5):
            pe_array[5][i].pe = Sink('north')
            pe_array[5][i].pe.build()
            pe_array[5][i].bound = pe_array[5][i].pe

        # Build ComputePEs
        for i in range(1, 5):
            for j in range(1, 5):
                if pe_array[i][j+1].bound is None:
                    print(f"Error: pe_array[{i}][{j+1}].bound is None")
                if pe_array[i+1][j].bound is None:
                    print(f"Error: pe_array[{i+1}][{j}].bound is None")
                fwest, fnorth = pe_array[i][j].pe.build(pe_array[i][j+1].bound, pe_array[i+1][j].bound)
                pe_array[i][j+1].bound = fwest
                pe_array[i+1][j].bound = fnorth

        # Build the SRAM module
        memory_R = SRAM_R(init_file_row, width)
        memory_R.wait_until()
        memory_R.build(width, \
                       pe_array[0][1].pe, \
                       pe_array[0][2].pe, \
                       pe_array[0][3].pe, \
                       pe_array[0][4].pe)
        
        memory_C = SRAM_C(init_file_col, width)
        memory_C.wait_until()
        memory_C.build(width, \
                       pe_array[1][0].pe, \
                       pe_array[2][0].pe, \
                       pe_array[3][0].pe, \
                       pe_array[4][0].pe)
        
        # Build the driver
        driver = Driver()
        driver.build(memory_R, memory_C)

    config = backend.config(sim_threshold=20, idle_threshold=20, resource_base=resource_base)

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    print(raw)
    check_raw(raw)
    # utils.run_verilator(verilator_path)

if __name__ == '__main__':
    mem_systolic_array('memory', 128, 'matrix_row.hex', 'matrix_col.hex', f'{utils.repo_path()}/examples/systolic-array/resource')
