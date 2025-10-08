import pytest
import re
from assassyn import backend
from assassyn.dtype import *
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.expr import Bind
from systolic_array import ProcElem, Sink, Pusher, check_raw

#  # PE Array (4 + 1) x (4 + 1)
#           [Pusher]      [Pusher]      [Pusher]      [Pusher]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#           [Sink]        [Sink]        [Sink]        [Sink]

class ComputePE(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={'west': Port(Int(8)), 'north': Port(Int(8))})
        self.acc = RegArray(Int(32), 1)

    @module.combinational
    def build(self, east: Bind, south: Bind):
        west, north = self.pop_all_ports(False)
        acc = self.acc
        val = acc[0]
        mul = (west * north)
        mul = concat(Bits(3)(0), mul)
        c = mul.bitcast(Int(32))
        mac = val + c
        log("Mac value: {} * {} + {} = {}", west, north, val, mac)
        acc[0] = mac

        res_east = east.bind(west = west)
        res_east.set_fifo_depth(west = 1)
        res_south = south.bind(north = north)
        res_south.set_fifo_depth(north = 1)
        if res_east.is_fully_bound():
            res_east = res_east.async_called()
        if res_south.is_fully_bound():
            res_south = res_south.async_called()

        return res_east, res_south
    
    def clear_and_pop_acc(self):
        """Clear the accumulator and return its current value."""
        current_value = self.acc[0]
        self.acc[0] = Int(32)(0)  # Reset acc to 0
        return current_value

class Distributor(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={'rdata': Port(Bits(32))})

    @module.combinational
    def build(self, row_pusher, col_pusher):

        width = 32
        sm_ity = Int(8)
        cnt = RegArray(sm_ity, 1, initializer=[0]) 
        buffer1 = RegArray(Int(width), 4)
        buffer2 = RegArray(Int(width), 4)

        sm_cnt = cnt[0]
        cnt[0] = sm_cnt + sm_ity(1)

        self.timing = 'systolic'

        buffer_idx = [slice(i * 8, i * 8 + 7) for i in range(3, -1, -1)]
        rdata = self.rdata.pop()
        rdata = rdata.bitcast(Int(32))

        with Condition(sm_cnt == sm_ity(0)):  
            for i in range(4):
                buffer1[i] = rdata

        with Condition(sm_cnt == sm_ity(1)): 
            for i in range(4):
                buffer2[i] = rdata

        for i in range(4):
            with Condition(sm_cnt == sm_ity(i + 2)): 
                for p_idx, b_idx in zip(range(4), reversed(range(4))):
                    row_pusher[p_idx].async_called(
                        data=buffer1[p_idx][buffer_idx[b_idx]].bitcast(Int(8))
                    )
                    col_pusher[p_idx].async_called(
                        data=buffer2[p_idx][buffer_idx[b_idx]].bitcast(Int(8))
                    )

        for i in range(4, 8):
            with Condition(sm_cnt == sm_ity(i + 2)):
                for p_idx, b_idx in zip(range(4), reversed(range(4))):
                    row_pusher[p_idx].async_called(
                        data=buffer1[p_idx][buffer_idx[b_idx]].bitcast(Int(8))
                    )
                    col_pusher[p_idx].async_called(
                        data=buffer2[p_idx][buffer_idx[b_idx]].bitcast(Int(8))
                    )

        with Condition(sm_cnt == sm_ity(10)):  
            cnt[0] = sm_ity(0) 
            for i in range(4):
                buffer1[i] = Int(width)(0)
                buffer2[i] = Int(width)(0)

class Driver(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={})

    @module.combinational
    def build(self, memory: SRAM, distributor: Distributor, pe_array, row_size, state, loop_cnt, cnt_read, cnt_compute):
        # State Machine Transitions

        # State 0: Start
        with Condition(state[0] == Int(8)(0)):
            with Condition(loop_cnt[0] < Int(32)(256)):
                state[0] = Int(8)(1)  # Transition to Read State
            with Condition(loop_cnt[0] >= Int(32)(256)):
                state[0] = Int(8)(4)  # Transition to Done State

        # State 1: Read Data
        with Condition(state[0] == Int(8)(1)):
            with Condition(Int(32)(cnt_read[0]) < Int(32)(8)):
                cnt_read[0] = Int(32)(cnt_read[0]) + Int(32)(1)  # Increment Read Counter
                
                # Address Calculation for Read Phase
                with Condition((Int(32)(cnt_read[0]) % Int(32)(2)) == Int(32)(0)):
                    # Even cnt_read: Address increments from 0 to 1023 (Matrix A)
                    raddr = (Int(32)(cnt_read[0]) * Int(32)(4) + (Int(32)(cnt_read[0]) >> 1))
                with Condition((Int(32)(cnt_read[0]) % Int(32)(2)) != Int(32)(0)):
                    # Odd cnt_read: Address increments from 1024 to 2048 (Matrix B)
                    raddr = (Int(32)(1024) + loop_cnt[0] * Int(32)(4) + (Int(32)(cnt_read[0]) >> 1))
                
                re = True # Read Enable Signal
                we = False # Write Disable Signal
            
            memory.build(we, re, raddr, Bits(memory.width)(0))
            distributor.async_called()

            with Condition(Int(32)(cnt_read[0]) == Int(32)(8)):
                cnt_read[0] = Int(32)(0)               # Reset Read Counter
                state[0] = Int(8)(2)                   # Transition to Compute State
        
        # State 2 Compute
        with Condition(state[0] == Int(8)(2)):
            with Condition(cnt_compute[0] < Int(32)(8)):
                cnt_compute[0] = cnt_compute[0] + Int(32)(1)
                # memory.bound.async_called()
                distributor.async_called()
                log("Distributor invoked!")
            
            with Condition(cnt_compute[0] == Int(32)(8)):
                cnt_compute[0] = Int(32)(0)            # Reset compute Counter
                state[0] = Int(8)(3)

        # State 3 Write-Back
        with Condition(state[0] == Int(8)(3)):
            with Condition(cnt_compute[0] < Int(32)(16)): # 16 x write back since acc is 32-bit now
                cnt_compute[0] = cnt_compute[0] + Int(32)(1)

                re = False # Read Enable Signal
                we = True  # Write Disable Signal

                i = ((cnt_compute[0] - Int(32)(1)) // Int(32)(array_size)) + Int(32)(1)
                j = ((cnt_compute[0] - Int(32)(1)) % Int(32)(array_size)) + Int(32)(1)
               
                # TODO wdata 
                # Clear and retrieve the accumulator value
                acc_value = pe_array[i][j].pe.clear_and_pop_acc()
                log("Pop accumulator value : {acc_value}")

                waddr = (Int(32)(2048) + 16*loop_cnt[0] + cnt_compute[0])
                memory.build(we, re, waddr, acc_value, distributor)
                
            with Condition(cnt_compute[0] == Int(32)(16)):
                cnt_compute[0] = Int(32)(0)            # Reset Compute Counter
                loop_cnt[0] = loop_cnt[0] + Int(32)(1)
                with Condition(loop_cnt[0] < Int(32)(256)):
                    state[0] = Int(8)(1)  # Loop back to Read State
                with Condition(loop_cnt[0] >= Int(32)(256)):
                    state[0] = Int(8)(4)  # Transition to Done State

        # State 4: Done
        with Condition(state[0] == Int(8)(4)):
            pass  # Remain in Done State
        
def build_pe_array(sys, array_size):
    res = [[ProcElem() for _ in range(array_size + 2)] for _ in range(array_size + 2)]

    # Init ComputePE
    for i in range(1, array_size + 1):
        for j in range(1, array_size + 1):
            res[i][j].pe = ComputePE()
            res[i][j].pe.name = f'pe_{i}_{j}'

    for i in range(1, array_size + 1):
        for j in range(1, array_size + 1):
            res[i][j].bound = res[i][j].pe

    for i in range(1, array_size + 1):
        # Build Column Pushers
        row_pusher = Pusher('row', i)
        col_pusher = Pusher('col', i)
        res[i][0].pe = row_pusher
        res[0][i].pe = col_pusher

        # First Row Pushers
        res[i][1].bound = row_pusher.build('west', res[i][1].bound)
        res[1][i].bound = col_pusher.build('north', res[1][i].bound)

        # Last Column Sink
        res[i][array_size + 1].pe = Sink('west')
        res[i][array_size + 1].pe.build()
        res[i][array_size + 1].bound = res[i][array_size + 1].pe

        # Last Row Sink
        res[array_size + 1][i].pe = Sink('north')
        res[array_size + 1][i].pe.build()
        res[array_size + 1][i].bound = res[array_size + 1][i].pe

    # Build ComputePEs
    for i in range(1, array_size + 1):
        for j in range(1, array_size + 1):
            fwest, fnorth = res[i][j].pe.build(
                    res[i][j + 1].bound,
                    res[i + 1][j].bound)
            sys.expose_on_top(res[i][j].pe.acc, kind='Inout')
            res[i][j + 1].bound = fwest
            res[i + 1][j].bound = fnorth

    return res
   
def mem_systolic_array(sys_name, init_file, array_size, row_size, resource_base):
    sys = SysBuilder(sys_name)

    with sys:
        
        # Build the PE array
        pe_array = build_pe_array(sys, array_size)
        
        # Build the SRAM module
        memory = SRAM(width=32, depth=10000, init_file=init_file) 

        # Build the Distributor
        distributor = Distributor()

        # Initialize Driver Registers
        state = RegArray(Int(8), 1, initializer=[0])          # State register
        loop_cnt = RegArray(Int(32), 1, initializer=[0])      # Loop counter
        cnt_read = RegArray(Int(32), 1, initializer=[0])      # Read counter
        cnt_compute = RegArray(Int(32), 1, initializer=[0])   # Phase counter (used for both compute and write-back)

        # Build the driver
        driver = Driver()
        driver.build(memory, distributor, pe_array, row_size, state, loop_cnt, cnt_read, cnt_compute)
        
        distributor.build(
            [pe_array[0][i].pe for i in range(1, array_size + 1)], 
            [pe_array[i][0].pe for i in range(1, array_size + 1)])

    config = backend.config(sim_threshold=2000, idle_threshold=2000, resource_base=resource_base)

    simulator_path, verilator_path = backend.elaborate(sys, **config)

    raw = utils.run_simulator(simulator_path)
    # print(raw)
    # check_raw(raw, 4)
    # raw = utils.run_verilator(verilator_path)
    print(raw)

if __name__ == '__main__':
    array_size = 4
    row_size = 64
    mem_systolic_array('systolic_w_memory_64', 'matrix_64.hex', array_size, row_size, f'{utils.repo_path()}/examples/systolic-array/resource')
