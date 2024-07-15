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
    def __init__(self):
        super().__init__()
        self._v = Port(Int(32))

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

def systolic_array():
    sys = SysBuilder('systolic-array')
    pe_array = [[ProcElem() for _ in range(6)] for _ in range(6)]
    
    with sys:
        # Column-wise Sinker
        for i in range(1, 5):
            pe_array[i][5].pe = Sink()
            pe_array[i][5].pe.build()
            pe_array[i][5].bound = pe_array[i][5].pe

        # Row-wise Sinker
        for i in range(1, 5):
            pe_array[5][i].pe = Sink()
            pe_array[5][i].pe.build()
            pe_array[5][i].bound = pe_array[5][i].pe

        for i in range(4, 0, -1):
            for j in range(4, 0, -1):
                peeast = pe_array[i][j + 1].pe
                fsouth = pe_array[i + 1][j].bound
                pe = ComputePE()
                print(f'i = {i}; j = {j}')
                feast, acc = pe.build(peeast, fsouth)
                #TODO: Set name of each ComputePE
                pe_array[i][j].pe = pe
                pe_array[i][j].bound = pe
                pe_array[i][j + 1].bound = feast
                pe_array[i][j].accumulator = acc
                pe_array[i + 1][j].bound = fsouth


    print(sys)

if __name__ == '__main__':
    systolic_array()
