import pytest
import re
from assassyn import backend
from assassyn.dtype import *
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.expr import Bind
from systolic_array import ProcElem, Sink, Pusher, ComputePE, check_raw, build_pe_array

#  # PE Array (4 + 1) x (4 + 1)
#           [Pusher]      [Pusher]      [Pusher]      [Pusher]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#           [Sink]        [Sink]        [Sink]        [Sink]

class Distributor(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={'rdata': Port(Bits(32))})

    @module.combinational
    def build(self, pushers):

        width = 32

        sm_ity = Int(8)
        cnt = RegArray(sm_ity, 1, initializer=[0])
        buffer = RegArray(Int(width), 4)

        sm_cnt = cnt[0]

        cnt[0] = sm_cnt + sm_ity(1)

        cond = sm_cnt < sm_ity(4)

        self.timing = 'systolic'

        buffer_idx = [slice(i * 8, i * 8 + 7) for i in range(3, -1, -1)]
        print(buffer_idx)

        buffer_related = []

        with Condition(cond):
            rdata = self.rdata.pop()
            rdata = rdata.bitcast(Int(32))

            for i in range(4):
                with Condition(sm_cnt == sm_ity(i)):
                    buffer[i] = rdata
                    for p_idx, b_idx in zip(buffer_related, reversed(buffer_related)):
                        pushers[p_idx].async_called(data = buffer[p_idx][buffer_idx[b_idx]].bitcast(Int(8)))
                buffer_related.append(i)

        for i in range(4, 8):
            with Condition(sm_cnt == sm_ity(i)):
                for p_idx, b_idx in zip(buffer_related, reversed(buffer_related)):
                    pushers[p_idx].async_called(data = buffer[p_idx][buffer_idx[b_idx]].bitcast(Int(8)))
            buffer_related = buffer_related[1:]

class Driver(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={})

    @module.combinational
    def build(self, memory_R: SRAM, memory_C: SRAM):
        cnt = RegArray(Int(8), 1, initializer=[0])
        v = cnt[0]
        raddr = v[0:9]  
        addr = raddr.bitcast(Int(10)) 
        re = cnt[0] < Int(8)(4)
        compute = cnt[0] < Int(8)(8)

        memory_R.build(Int(1)(0), re, addr, Bits(memory_R.width)(0))
        memory_C.build(Int(1)(0), re, addr, Bits(memory_C.width)(0))

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

        pe_array = build_pe_array(sys, 4)

        # Build the SRAM module
        memory_R = SRAM(width=32, depth=1024, init_file=init_file_row) 
        memory_C = SRAM(width=32, depth=1024, init_file=init_file_col)

        # Build the Distributor
        rd = Distributor()
        cd = Distributor()

        # Build the driver
        driver = Driver()
        compute = driver.build(memory_R, memory_C, rd, cd)
        rd.build([pe_array[0][i].pe for i in range(1, 5)])
        cd.build([pe_array[i][0].pe for i in range(1, 5)])

        # Invoke the Distributor
        invoker = Invoker()
        invoker.build(rd, cd, compute)

    config = backend.config(sim_threshold=20, idle_threshold=20, resource_base=resource_base)

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    print(raw)
    check_raw(raw, 4)
    # utils.run_verilator(verilator_path)

if __name__ == '__main__':
    mem_systolic_array('systolic_w_memory', 'matrix_row.hex', 'matrix_col.hex', f'{utils.repo_path()}/examples/systolic-array/resource')
