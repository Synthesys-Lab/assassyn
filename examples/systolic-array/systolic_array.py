import pytest
import re
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn import utils
from assassyn.expr import Bind

#  # PE Array (4 + 1) x (4 + 1)
#           [Pusher]      [Pusher]      [Pusher]      [Pusher]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#           [Sink]        [Sink]        [Sink]        [Sink]

class ProcElem():
    def __init__(self, pe=None, bound=None):
        self.pe = pe
        self.bound = bound

class Sink(Module):
    
    def __init__(self, port_name='_v'):
        super().__init__(no_arbiter=True, ports={port_name: Port(Int(8))})

    @module.combinational
    def build(self):
        data = self.pop_all_ports(False)
        log("Sink: {}", data)

class ComputePE(Module):

    def __init__(self):
        super().__init__(no_arbiter=True, ports={'west': Port(Int(8)), 'north': Port(Int(8))})
        self.acc = RegArray(Int(19), 1)

    @module.combinational
    def build(self, east: Bind, south: Bind):
        west, north = self.pop_all_ports(False)
        acc = self.acc
        val = acc[0]
        mul = (west * north)
        mul = concat(Bits(3)(0), mul)
        c = mul.bitcast(Int(19))
        mac = val + c
        log("Mac value: {} * {} + {} = {}", west, north, val, mac)
        acc[0] = mac

        res_east = east.bind(west = west)
        res_south = south.bind(north = north)
        if res_east.is_fully_bound():
            res_east = res_east.async_called()
        if res_south.is_fully_bound():
            res_south = res_south.async_called()

        return res_east, res_south

class Pusher(Module):

    def __init__(self, prefix, idx):
        super().__init__(no_arbiter=True, ports={'data': Port(Int(8))})
        self.name = f'{prefix}_Pusher_{idx}'

    @module.combinational
    def build(self, direction: str, dest: Bind):
        data = self.pop_all_ports(False)
        log(f"{self.name} pushes {{}}", data)
        kwargs = {direction: data}
        new_bind = dest.bind(**kwargs)
        if new_bind.is_fully_bound():
            res = new_bind.async_called()
            return res
        return new_bind

class Testbench(Module):
    
    def __init__(self):
        super().__init__(ports={}, no_arbiter=True)

    @module.combinational
    def build(self, rows, cols, array_size):

        def build_call(x, data):
            for row, col, data in zip(rows[x], cols[x], data):
                row.async_called(data = Int(8)(data))
                col.async_called(data = Int(8)(data))


        # Example:
        # array_size = 4
        
        # Cycle 1:
        # 1 0
        # 0 P P P  P
        #   P P P  P
        #   P P P  P
        #   P P P  P        
        # build_call(slice(0, 1), [0])

        # Cycle 2:
        # 2 1 4
        # 1 P P P  P
        # 4 P P P  P
        #   P P P  P
        #   P P P  P            
        # build_call(slice(0, 2), [1, 4])

        # Cycle 3:
        # 3 2 5 8
        # 2 P P P  P
        # 5 P P P  P
        # 8 P P P  P
        #   P P P  P
        # build_call(slice(0, 3), [2, 5, 8])

        # Cycle 4:
        # 4  3 6 9  12
        # 3  P P P  P
        # 6  P P P  P
        # 9  P P P  P
        # 12 P P P  P
        # build_call(slice(0, 4), [3, 6, 9, 12])
        
        # Cycle 5:
        # 5    7 10 13
        #    P P P  P
        # 7  P P P  P
        # 10 P P P  P
        # 13 P P P  P            
        # build_call(slice(1, 4), [7, 10, 13])

        # Cycle 6:
        #  6    11 14
        #    P P P  P
        #    P P P  P
        # 11 P P P  P
        # 14 P P P  P
        # build_call(slice(2, 4), [11, 14])
            
        # Cycle 7:
        #   7      15
        #    P P P  P
        #    P P P  P
        #    P P P  P
        # 15 P P P  P
        # build_call(slice(3, 4), [15])

        for i in range(1, array_size + 1):
            slice_range = slice(0, i)
            values = [j * array_size + (i - j - 1) for j in range(i)
                        if j * array_size + (i - j - 1) < array_size * array_size]

            with Cycle(i):
                build_call(slice_range, values)

        for i in range(array_size + 1, 2 * array_size):
            slice_range = slice(i - array_size, array_size)
            values = [(i - array_size + j) * array_size + (array_size - j - 1)
                        for j in range(2 * array_size - i)
                        if (i - array_size + j) * array_size + (array_size - j - 1) < array_size * array_size]

            with Cycle(i):
                build_call(slice_range, values)

def check_raw(raw, array_size):
    a = [[0 for _ in range(array_size)] for _ in range(array_size)]
    b = [[0 for _ in range(array_size)] for _ in range(array_size)]
    c = [[0 for _ in range(array_size)] for _ in range(array_size)]
    
    for i in range(array_size):
        for j in range(array_size):
            a[i][j] = i * array_size + j
            b[j][i] = i * array_size + j
    
    for i in range(array_size):
        for j in range(array_size):
            for k in range(array_size):
                c[i][j] += a[i][k] * b[k][j]
    
    for i in range(array_size):
        for j in range(array_size):
            expected = c[i][j]
            pattern = rf"pe_{i+1}_{j+1}"
            matching_lines = [line for line in raw.split('\n') if re.search(pattern, line)]
            if matching_lines:
                actual_line = matching_lines[-1]  
                print(actual_line)
                actual = int(actual_line.split()[-1])
                assert expected == actual

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
            sys.expose_on_top(res[i][j].pe.acc, kind='Output')
            res[i][j + 1].bound = fwest
            res[i + 1][j].bound = fnorth

    return res


def systolic_array(array_size):
    sys = SysBuilder('systolic_array')
    
    with sys:
        pe_array = build_pe_array(sys, array_size)
        testbench = Testbench()
        testbench.build(
                [pe_array[0][i].pe for i in range(1, array_size + 1)], \
                [pe_array[i][0].pe for i in range(1, array_size + 1)],
                array_size)

    simulator_path, verilator_path = elaborate(sys, verilog="verilator")

    raw = utils.run_simulator(simulator_path)
    check_raw(raw, array_size)

    raw = utils.run_verilator(verilator_path)
    check_raw(raw, array_size)

if __name__ == '__main__':
    array_size = 8
    systolic_array(array_size)
