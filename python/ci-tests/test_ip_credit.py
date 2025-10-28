"""Test case for credit counter IP module.

Tests the CreditCounter downstream module by generating delta increments
and pop decrements, verifying the counter logic matches expected behavior.
"""

from assassyn.frontend import *
from assassyn.test import run_test
from assassyn.ip.credit import CreditCounter

class Driver(Module):
    """Driver module that generates test patterns for credit counter."""
    
    def __init__(self):
        super().__init__(ports={})
    
    @module.combinational
    def build(self, counter_downstream: Downstream):
        cycle = RegArray(UInt(32), 1)
        (cycle & self)[0] <= cycle[0] + UInt(32)(1)
        
        # Generate delta: cycles through 0, 1, 2 (never exceeds capacity)
        # With max count of 8 bits (255) and pop every 4 cycles, this pattern
        # ensures we add at most 2+2+2 = 6 per 4-cycle window while popping 1,
        # net gain of 5 per window, taking 51 windows to reach 255, well over 100 cycles
        delta = (cycle[0] % UInt(32)(3)).bitcast(UInt(8))
        
        # Pop every 4th cycle to prevent overflow
        pop = (cycle[0] % UInt(32)(4) == UInt(32)(0)).bitcast(UInt(1))
        
        # Call credit counter downstream
        delta_ready, pop_valid = counter_downstream.build(delta, pop)

def check_raw(raw):
    """Verify credit counter behavior with strict checking."""
    expected_count = 0
    cnt = 0
    max_val = 255  # 8-bit counter max
    
    for line in raw.split('\n'):
        if 'CreditCounter:' in line:
            # Parse: CreditCounter: count=X delta=Y pop=Z -> new_count=W ready=R valid=V
            # Extract the log part after "CreditCounter: "
            log_part = line.split('CreditCounter: ')[1]
            parts = log_part.split()
            
            # Parse: count=X delta=Y pop=Z -> new_count=W ready=R valid=V
            count = int(parts[0].split('=')[1])
            delta = int(parts[1].split('=')[1])
            pop = int(parts[2].split('=')[1])
            new_count = int(parts[4].split('=')[1])  # After "->"
            
            # Verify logic: new_count = count + delta - pop (with underflow protection)
            expected = count + delta - pop
            if expected < 0:
                expected = 0
            if expected > max_val:  # Should never overflow with our pattern
                expected = max_val
            
            assert count == expected_count, \
                f'Count mismatch at cycle {cnt}: got {count}, expected {expected_count}'
            assert new_count == expected, \
                f'New count mismatch at cycle {cnt}: got {new_count}, expected {expected}'
            assert new_count <= max_val, \
                f'Overflow at cycle {cnt}: new_count={new_count} > {max_val}'
            
            expected_count = expected
            cnt += 1
    
    assert cnt >= 90, f'Expected ~100 cycles, got {cnt}'

def build_system():
    """Build the test system with credit counter and driver."""
    counter = CreditCounter(width=8, debug=True)
    driver = Driver()
    driver.build(counter)

def test_ip_credit():
    """Run the credit counter IP test."""
    run_test('ip_credit', build_system, check_raw, 
             sim_threshold=100, idle_threshold=200, verilog=True)

if __name__ == '__main__':
    test_ip_credit()
