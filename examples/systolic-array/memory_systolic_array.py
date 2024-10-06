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

class SRAM_R(SRAM):

    def __init__(self, init_file, width):
        super().__init__(width=width, depth=1024, init_file=init_file)

class RDistributor(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={'rdata': Port(Bits(128))})

    @module.combinational
    def build(self, row1: RowPusher, row2: RowPusher, row3: RowPusher, row4: RowPusher):

        width = 128

        sm_ity = Int(8)
        cnt = RegArray(sm_ity, 1, initializer=[0])
        buffer = RegArray(Int(width), 4)

        sm_cnt = cnt[0]

        cnt[0] = sm_cnt + sm_ity(1)

        cond = sm_cnt < sm_ity(4)

        self.timing = 'systolic'

        with Condition(cond):
            rdata = self.rdata.pop()
            rdata = rdata.bitcast(Int(128))

            with Condition(sm_cnt == sm_ity(0)):
                buffer[0] = rdata

            with Condition(sm_cnt == sm_ity(1)):
                buffer[1] = rdata
                row1.async_called(data = buffer[0][96:127].bitcast(Int(32)))
                log("Row Buffer data (hex): {:#034x}", buffer[0]) 

            with Condition(sm_cnt == sm_ity(2)):
                buffer[2] = rdata
                row1.async_called(data = buffer[0][64:95].bitcast(Int(32)))
                row2.async_called(data = buffer[1][96:127].bitcast(Int(32)))
                log("Row Buffer data (hex): {:#034x}", buffer[1]) 

            with Condition(sm_cnt == sm_ity(3)):
                buffer[3] = rdata
                row1.async_called(data = buffer[0][32:63].bitcast(Int(32)))
                row2.async_called(data = buffer[1][64:95].bitcast(Int(32)))
                row3.async_called(data = buffer[2][96:127].bitcast(Int(32)))
                log("Row Buffer data (hex): {:#034x}", buffer[2]) 

        with Condition(sm_cnt == sm_ity(4)):
            log("Row Buffer data (hex): {:#034x}", buffer[3])
            row1.async_called(data = buffer[0][0:31].bitcast(Int(32)))
            row2.async_called(data = buffer[1][32:63].bitcast(Int(32)))
            row3.async_called(data = buffer[2][64:95].bitcast(Int(32)))
            row4.async_called(data = buffer[3][96:127].bitcast(Int(32)))

        with Condition(sm_cnt == sm_ity(5)):
            row2.async_called(data = buffer[1][0:31].bitcast(Int(32)))
            row3.async_called(data = buffer[2][32:63].bitcast(Int(32)))
            row4.async_called(data = buffer[3][64:95].bitcast(Int(32)))

        with Condition(sm_cnt == sm_ity(6)):
            row3.async_called(data = buffer[2][0:31].bitcast(Int(32)))
            row4.async_called(data = buffer[3][32:63].bitcast(Int(32)))

        with Condition(sm_cnt == sm_ity(7)):
            row4.async_called(data = buffer[3][0:31].bitcast(Int(32)))


class SRAM_C(SRAM):

    def __init__(self, init_file, width):
        super().__init__(width=width, depth=1024, init_file=init_file)


class CDistributor(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={'rdata': Port(Bits(128))})

    @module.combinational
    def build(self, col1: ColPusher, col2: ColPusher, col3: ColPusher, col4: ColPusher):

        width = 128
        sm_ity = Int(8)
        buffer = RegArray(Int(width), 4)

        cnt = RegArray(sm_ity, 1, initializer=[0])
        sm_cnt = cnt[0]
        cnt[0] = sm_cnt + sm_ity(1)

        self.timing = 'systolic'

        with Condition(sm_cnt < sm_ity(4)):
            rdata = self.rdata.pop()
            rdata = rdata.bitcast(Int(128))


            with Condition(sm_cnt == sm_ity(0)):
                buffer[0] = rdata

            with Condition(sm_cnt == sm_ity(1)):
                buffer[1] = rdata
                col1.async_called(data = buffer[0][96:127].bitcast(Int(32)))
                log("Col Buffer data (hex): {:#034x}", buffer[0]) 

            with Condition(sm_cnt == sm_ity(2)):
                buffer[2] = rdata
                col1.async_called(data = buffer[0][64:95].bitcast(Int(32)))
                col2.async_called(data = buffer[1][96:127].bitcast(Int(32)))
                log("Col Buffer data (hex): {:#034x}", buffer[1]) 

            with Condition(sm_cnt == sm_ity(3)):
                buffer[3] = rdata
                col1.async_called(data = buffer[0][32:63].bitcast(Int(32)))
                col2.async_called(data = buffer[1][64:95].bitcast(Int(32)))
                col3.async_called(data = buffer[2][96:127].bitcast(Int(32)))
                log("Col Buffer data (hex): {:#034x}", buffer[2]) 

        with Condition(sm_cnt == sm_ity(4)):
            log("Col Buffer data (hex): {:#034x}", buffer[3])
            col1.async_called(data = buffer[0][0:31].bitcast(Int(32)))
            col2.async_called(data = buffer[1][32:63].bitcast(Int(32)))
            col3.async_called(data = buffer[2][64:95].bitcast(Int(32)))
            col4.async_called(data = buffer[3][96:127].bitcast(Int(32)))

        with Condition(sm_cnt == sm_ity(5)):
            col2.async_called(data = buffer[1][0:31].bitcast(Int(32)))
            col3.async_called(data = buffer[2][32:63].bitcast(Int(32)))
            col4.async_called(data = buffer[3][64:95].bitcast(Int(32)))

        with Condition(sm_cnt == sm_ity(6)):
            col3.async_called(data = buffer[2][0:31].bitcast(Int(32)))
            col4.async_called(data = buffer[3][32:63].bitcast(Int(32)))

        with Condition(sm_cnt == sm_ity(7)):
            col4.async_called(data = buffer[3][0:31].bitcast(Int(32)))



class Driver(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={})

    @module.combinational
    def build(self, memory_R: SRAM_R, memory_C: SRAM_C, rd: RDistributor, cd: CDistributor):
        cnt = RegArray(Int(8), 1, initializer=[0])
        v = cnt[0]
        raddr = v[0:9]  
        addr = raddr.bitcast(Int(10)) 
        re = cnt[0] < Int(8)(4)
        compute = cnt[0] < Int(8)(8)

        memory_R.build(Int(1)(0), re, addr, Bits(memory_R.width)(0), rd)
        memory_C.build(Int(1)(0), re, addr, Bits(memory_C.width)(0), cd)

        cnt[0] = cnt[0] + Int(8)(1)

        return compute



class Invoker(Downstream):

    def __init__(self):
        super().__init__()


    @downstream.combinational
    def build(self, rd, cd, compute):
        with Condition(compute):
            rd.async_called()
            cd.async_called()
            log("Distributor invoked!")

        
def mem_systolic_array(sys_name, init_file_row, init_file_col, resource_base):
    sys = SysBuilder(sys_name)
    pe_array = [[ProcElem() for _ in range(6)] for _ in range(6)]

    with sys:

        # Init ComputePE
        for i in range(1, 5):
            for j in range(1, 5):
                pe_array[i][j].pe = ComputePE()
                pe_array[i][j].pe.name = f"ComputePE_{i}_{j}"

        for i in range(1, 5):
            for j in range(1, 5):
                pe_array[i][j].bound = pe_array[i][j].pe

        # First Column Pushers
        for i in range(1, 5):
            pe_array[i][0].pe = ColPusher()
            pe_array[i][0].pe.name = f"ColPusher_{i}"
            if pe_array[i][1].bound is not None:  
                bound = pe_array[i][0].pe.build(pe_array[i][1].bound)
                pe_array[i][0].bound = bound
            else:
                print(f"Error: pe_array[{i}][1].bound is not initialized!")

        # First Row Pushers
        for i in range(1, 5):
            pe_array[0][i].pe = RowPusher()
            pe_array[0][i].pe.name = f"RowPusher_{i}"
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
        memory_R = SRAM_R(init_file_row, 128)
        memory_C = SRAM_C(init_file_col, 128)

        # Build the Distributor
        rd = RDistributor()
        cd = CDistributor()

        # Build the driver
        driver = Driver()
        compute = driver.build(memory_R, memory_C, rd, cd)
        rd.build(*[pe_array[0][i].pe for i in range(1, 5)])
        cd.build(*[pe_array[i][0].pe for i in range(1, 5)])

        # Invoke the Distributor
        invoker = Invoker()
        invoker.build(memory_R.bound, memory_C.bound, compute)

    config = backend.config(sim_threshold=20, idle_threshold=20, resource_base=resource_base)

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    print(raw)
    check_raw(raw)
    # utils.run_verilator(verilator_path)

if __name__ == '__main__':
    mem_systolic_array('systolic_w_memory', 'matrix_row.hex', 'matrix_col.hex', f'{utils.repo_path()}/examples/systolic-array/resource')
