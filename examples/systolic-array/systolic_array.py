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
    
    @module.constructor
    def __init__(self, port_name='_v'):
        super().__init__()
        setattr(self, port_name, Port(Int(32)))
        self.port_name = port_name

    @module.combinational
    def build(self):
        log("Sink: {}", getattr(self, self.port_name))

class ComputePE(Module):

    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)
        self.west = Port(Int(32))
        self.north = Port(Int(32))

    @module.combinational
    def build(self, east: Bind, south: Bind):
        acc = RegArray(Int(32), 1)
        val = acc[0]
        mul = (self.west * self.north)
        c = mul[0:31].bitcast(Int(32))
        mac = val + c
        log("Mac value: {} * {} + {} = {}", self.west, self.north, val, mac)
        acc[0] = mac

        bound_east = east.bind(west = self.west)
        bound_south = south.bind(north = self.north)
        if bound_east.is_fully_bound():
            bound_east.async_called()
        if bound_south.is_fully_bound():
            bound_south.async_called()

        return bound_east, bound_south

class RowPusher(Module):

    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)
        self.data = Port(Int(32))

    @module.combinational
    def build(self, dest: Bind):
        log("Row Pushes {}", self.data)
        dest.async_called(north = self.data)

class ColPusher(Module):

    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)
        self.data = Port(Int(32))

    @module.combinational
    def build(self, dest: Bind):
        log("Col Pushes {}", self.data)
        dest.async_called(west = self.data)

class Testbench(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    # what if i do this?
    # Cycle:
    #    7                    15
    #      6               11 14
    #        5           7 10 13
    #          4       3 6 9  12
    #            3     2 5 8
    #              2   1 4
    #                1 0
    #          3 2 1 0 P P P  P
    #        7 6 5 4   P P P  P
    #    11 10 9 8     P P P  P
    # 15 14 13 12      P P P  P#
    # row [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]
    # col [[0, 4, 8, 12], [1, 5, 9, 13], [2, 6, 10, 14], [3, 7, 11, 15]]

    @module.combinational
    def build(self, col1: ColPusher, col2: ColPusher, col3: ColPusher, col4: ColPusher, \
                    row1: RowPusher, row2: RowPusher, row3: RowPusher, row4: RowPusher):
        with Cycle(1):
            # 1 0
            # 0 P P P  P
            #   P P P  P
            #   P P P  P
            #   P P P  P        
            col1.async_called(data = Int(32)(0))
            row1.async_called(data = Int(32)(0))

        with Cycle(2):
            # 2 1 4
            # 1 P P P  P
            # 4 P P P  P
            #   P P P  P
            #   P P P  P            
            row1.async_called(data = Int(32)(1))
            col1.async_called(data = Int(32)(1))
            row2.async_called(data = Int(32)(4))
            col2.async_called(data = Int(32)(4))

        with Cycle(3):
            # 3 2 5 8
            # 2 P P P  P
            # 5 P P P  P
            # 8 P P P  P
            #   P P P  P
            row1.async_called(data = Int(32)(2))
            col1.async_called(data = Int(32)(2))
            row2.async_called(data = Int(32)(5))
            col2.async_called(data = Int(32)(5))
            row3.async_called(data = Int(32)(8))
            col3.async_called(data = Int(32)(8))

        with Cycle(4):
            # 4  3 6 9  12
            # 3  P P P  P
            # 6  P P P  P
            # 9  P P P  P
            # 12 P P P  P
            row1.async_called(data = Int(32)(3))
            col1.async_called(data = Int(32)(3))
            row2.async_called(data = Int(32)(6))
            col2.async_called(data = Int(32)(6))
            row3.async_called(data = Int(32)(9))
            col3.async_called(data = Int(32)(9))
            row4.async_called(data = Int(32)(12))
            col4.async_called(data = Int(32)(12))
        
        with Cycle(5):
            # 5    7 10 13
            #    P P P  P
            # 7  P P P  P
            # 10 P P P  P
            # 13 P P P  P            
            row2.async_called(data = Int(32)(7))
            col2.async_called(data = Int(32)(7))
            row3.async_called(data = Int(32)(10))
            col3.async_called(data = Int(32)(10))
            row4.async_called(data = Int(32)(13))
            col4.async_called(data = Int(32)(13))

        with Cycle(6):
            #  6    11 14
            #    P P P  P
            #    P P P  P
            # 11 P P P  P
            # 14 P P P  P
            row3.async_called(data = Int(32)(11))
            col3.async_called(data = Int(32)(11))
            row4.async_called(data = Int(32)(14))
            col4.async_called(data = Int(32)(14))
            
        with Cycle(7):
            #   7      15
            #    P P P  P
            #    P P P  P
            #    P P P  P
            # 15 P P P  P
            row4.async_called(data = Int(32)(15))
            col4.async_called(data = Int(32)(15))

def check_raw(raw):
    a = [[0 for _ in range(4)] for _ in range(4)]
    b = [[0 for _ in range(4)] for _ in range(4)]
    c = [[0 for _ in range(4)] for _ in range(4)]
    
    for i in range(4):
        for j in range(4):
            a[i][j] = i * 4 + j
            b[j][i] = i * 4 + j
    
    for i in range(4):
        for j in range(4):
            for k in range(4):
                c[i][j] += a[i][k] * b[k][j]
    
    for i in range(4):
        for j in range(4):
            expected = c[i][j]
            pattern = rf"pe_{i+1}_{j+1}"
            matching_lines = [line for line in raw.split('\n') if re.search(pattern, line)]
            if matching_lines:
                actual_line = matching_lines[-1]  
                print(actual_line)
                actual = int(actual_line.split()[-1])
                assert expected == actual

def systolic_array():
    sys = SysBuilder('systolic_array')
    pe_array = [[ProcElem() for _ in range(6)] for _ in range(6)]
    
    with sys:
        # Init ComputePE
        for i in range(1, 5):
            for j in range(1, 5):
                pe_array[i][j].pe = ComputePE()

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
        for i in range(4, 0, -1):
            for j in range(4, 0, -1):
                feast, fsouth = pe_array[i][j].pe.build(pe_array[i][j+1].bound, pe_array[i+1][j].bound)
                pe_array[i][j].bound = pe_array[i][j].pe
                pe_array[i][j+1].bound = feast
                pe_array[i+1][j].bound = fsouth

        # First Column Pushers
        for i in range(1, 5):
            pe_array[i][0].pe = ColPusher()
            bound = pe_array[i][0].pe.build(pe_array[i][1].bound)
            pe_array[i][0].bound = bound

        # First Row Pushers
        for i in range(1, 5):
            pe_array[0][i].pe = RowPusher()
            pe_array[0][i].pe.build(pe_array[1][i].bound)

        testbench = Testbench()
        testbench.build(pe_array[0][1].pe, \
                        pe_array[0][2].pe, \
                        pe_array[0][3].pe, \
                        pe_array[0][4].pe, \
                        pe_array[1][0].pe, \
                        pe_array[2][0].pe, \
                        pe_array[3][0].pe, \
                        pe_array[4][0].pe)

    simulator_path, verilator_path = elaborate(sys, verilog="verilator")

    raw = utils.run_simulator(simulator_path)
    check_raw(raw)
    print(raw)

    raw = utils.run_verilator(verilator_path)
    check_raw(raw)
    print(raw)

if __name__ == '__main__':
    systolic_array()
