import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils

#  # PE Array (4 + ?) x (4 + ?)

#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  
#  [Pusher] [Compute PE]  [Compute PE]  [Compute PE]  [Compute PE]  

class ComputePE(Module):

    @module.constructor
    def __init__(self):
        super().__init__(disable_arbiter_rewrite=True)
        self.input = Port(Int(32))
        self.output = Port(Int(32))

    @module.combinational
    def build(self, input: Bind, output: Bind):
        acc = RegArray(Int(32), 1)
        val = acc[0]
        mul = (self.west * self.north)
        c = mul[0:31].bitcast(Int(32))
        mac = val + c
        log("Mac value: {} * {} + {} = {}", self.west, self.north, val, mac)
        acc[0] = mac