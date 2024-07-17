import pytest
from dataclasses import dataclass
from assassyn.frontend import *
from assassyn.backend import elaborate
from assassyn.expr import Bind

#  # PE Array (4 + 1) x (4 + 1)
#           [Pusher]      [Pusher]      [Pusher]      [Pusher]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  [Sink]
#           [Sink]        [Sink]        [Sink]        [Sink]

class ProcElem():
    def __init__(self, pe=None, bound=None, accumulator=None):
        self.pe = pe
        self.bound = bound
        self.accumulator = accumulator

class Sink(Module):

    @module.constructor
    def __init__(self, port_name='_v'):
        super().__init__()
        setattr(self, port_name, Port(Int(32)))

    @module.combinational
    def build(self):
        log("Hello")

class ComputePE(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.west = Port(Int(32))
        self.north = Port(Int(32))

    @module.combinational
    def build(self, east: Bind, south: Bind):
        c = self.west * self.north
        acc = RegArray(Int(64), 1)
        val = acc[0]
        mac = val + c
        log("Mac value: {} * {} + {} = {}", self.west, self.north, val, mac)
        acc[0] = mac
        feast = east.bind(west=self.west)
        if feast.is_fully_bound():
            feast.async_called()
        south.async_called(north=self.north)
        return feast, acc

class RowPusher(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()
        self.data = Port(Int(32))

    @module.combinational
    def build(self, dest: ComputePE):
        log("pushes {}", self.data)
        bound = dest.bind(north=self.data)
        return bound

class ColPusher(Module):

    @module.constructor
    def __init__(self):
        super().__init__()
        self.data = Port(Int(32))

    @module.combinational
    def build(self, dest: Bind):
        log("pushes {}", self.data)
        dest.async_called(west=self.data)

class Testbench(Module):
    
    @module.constructor
    def __init__(self):
        super().__init__()

    # what if i do this?
    # Cycle:
    #    6                    15
    #      5               11 14
    #        4           7 10 13
    #          3       3 6 9  12
    #            2     2 5 8
    #              1   1 4
    #                0 0
    #          3 2 1 0 P P P  P
    #        7 6 5 4   P P P  P
    #    11 10 9 8     P P P  P
    # 15 14 13 12      P P P  P#
    # row [[0, 1, 2, 3], [4, 5, 6, 7], [8, 9, 10, 11], [12, 13, 14, 15]]
    # col [[0, 4, 8, 12], [1, 5, 9, 13], [2, 6, 10, 14], [3, 7, 11, 15]]

    @module.combinational
    def build(self, col1: ColPusher, col2: ColPusher, col3: ColPusher, col4: ColPusher, \
                    row1: RowPusher, row2: RowPusher, row3: RowPusher, row4: RowPusher):
        with Cycle(0):
            # 0 0
            # 0 P P P  P
            #   P P P  P
            #   P P P  P
            #   P P P  P        
            col1.async_called(data = Int(32)(0))
            row1.async_called(data = Int(32)(0))

        with Cycle(1):
            # 1 1 4
            # 1 P P P  P
            # 4 P P P  P
            #   P P P  P
            #   P P P  P            
            row1.async_called(data = Int(32)(1))
            col1.async_called(data = Int(32)(1))
            row2.async_called(data = Int(32)(4))
            col2.async_called(data = Int(32)(4))

        with Cycle(2):
            # 2 2 5 8
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

        with Cycle(3):
            # 3  3 6 9  12
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
        
        with Cycle(4):
            # 4    7 10 13
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

        with Cycle(5):
            #  5    11 14
            #    P P P  P
            #    P P P  P
            # 11 P P P  P
            # 14 P P P  P
            row3.async_called(data = Int(32)(11))
            col3.async_called(data = Int(32)(11))
            row4.async_called(data = Int(32)(14))
            col4.async_called(data = Int(32)(14))
            
        with Cycle(6):
            #   6      15
            #    P P P  P
            #    P P P  P
            #    P P P  P
            # 15 P P P  P
            row4.async_called(data = Int(32)(15))
            col4.async_called(data = Int(32)(15))

def systolic_array():
    sys = SysBuilder('systolic-array')
    pe_array = [[ProcElem() for _ in range(6)] for _ in range(6)]
    
    with sys:
        # Column-wise Sinker
        for i in range(1, 5):
            pe_array[i][5].pe = Sink('west')
            pe_array[i][5].pe.build()
            pe_array[i][5].bound = pe_array[i][5].pe

        # Row-wise Sinker
        for i in range(1, 5):
            pe_array[5][i].pe = Sink('north')
            pe_array[5][i].pe.build()
            pe_array[5][i].bound = pe_array[5][i].pe

        for i in range(4, 0, -1):
            for j in range(4, 0, -1):
                peeast = pe_array[i][j + 1].pe
                fsouth = pe_array[i + 1][j].bound
                pe = ComputePE()
                print(f'i = {i}; j = {j}')
                feast, acc = pe.build(peeast, fsouth)
                #TODO: Set name of each ComputePE to pe_{i}_{j}
                pe_array[i][j].pe = pe
                pe_array[i][j].bound = pe
                pe_array[i][j + 1].bound = feast
                pe_array[i][j].accumulator = acc
                pe_array[i + 1][j].bound = fsouth

            pusher_pe = RowPusher()
            bound = pusher_pe.build(pe_array[i][1].bound)
            # set the pushe_pe's name into row_pusher_{i}
            pe_array[i][0].pe = pusher_pe
            pe_array[i][1].bound = bound

        for i in range(1, 5):
            pusher_pe = ColPusher()
            pusher_pe.build(pe_array[1][i].bound)
            #TODO: set the pusher_pe's name into col_pusher_{i}
            pe_array[0][i].pe = pusher_pe

        testbench = Testbench()
        testbench.build(pe_array[0][1].pe, \
                        pe_array[0][2].pe, \
                        pe_array[0][3].pe, \
                        pe_array[0][4].pe, \
                        pe_array[1][0].pe, \
                        pe_array[2][0].pe, \
                        pe_array[3][0].pe, \
                        pe_array[4][0].pe)


    print(sys)

if __name__ == '__main__':
    systolic_array()
