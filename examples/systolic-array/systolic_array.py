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
    def __init__(self, pe, bound, accumulator):
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
        pass


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


def systolic_array():
    sys = SysBuilder('systolic-array')
