"""Credit counter IP module.

This module implements a credit counter equivalent to trigger_counter.sv,
designed for credit-based flow control in pipeline architectures.
"""

from assassyn.frontend import *

class CreditCounter(Downstream):
    """Credit counter downstream module.
    
    A combinational module that maintains an internal counter for credit-based
    flow control. The counter increments by delta and decrements when pop_ready
    is asserted, with overflow/underflow protection.
    
    Args:
        width: Bit width of the counter (default: 8)
        debug: Enable debug logging (default: False)
    
    Build Parameters:
        delta: Value to add to counter (UInt(width))
        pop_ready: Signal to decrement counter by 1 (UInt(1))
    
    Returns:
        Tuple of (delta_ready, pop_valid):
        - delta_ready: High when counter is not at maximum (can accept more credits)
        - pop_valid: High when counter is non-zero or delta is non-zero
    """
    
    def __init__(self, width: int = 8, debug: bool = False):
        super().__init__()
        self.width = width
        self.debug = debug
    
    @downstream.combinational
    def build(self, delta: Value, pop_ready: Value):
        # Internal counter register
        count_reg = RegArray(UInt(self.width), 1)
        count = count_reg[0]
        
        # Handle optional inputs
        delta = delta.optional(UInt(self.width)(0))
        pop_ready = pop_ready.optional(UInt(1)(0))
        
        # Compute new count: temp = count + delta - pop_ready
        temp = count + delta
        pop_amount = pop_ready.select(UInt(self.width)(1), UInt(self.width)(0))
        new_count = (temp >= pop_amount).select(temp - pop_amount, UInt(self.width)(0))
        
        # Update register
        count_reg[0] = new_count
        
        # Compute outputs
        max_val = UInt(self.width)((1 << self.width) - 1)
        delta_ready = new_count != max_val
        pop_valid = (new_count != UInt(self.width)(0)) | (delta != UInt(self.width)(0))
        
        # Debug logging
        if self.debug:
            log("CreditCounter: count={} delta={} pop={} -> new_count={} ready={} valid={}", 
                count, delta, pop_ready, new_count, delta_ready, pop_valid)
        
        return (delta_ready, pop_valid)
