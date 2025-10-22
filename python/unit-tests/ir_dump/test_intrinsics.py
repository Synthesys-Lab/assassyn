"""Tests for intrinsic IR nodes: Intrinsic, PureIntrinsic."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
    # Core types
    Module, SysBuilder, UInt, Int, Bits, Record,
    # Arrays and blocks
    RegArray, Condition, Cycle,
    # Intrinsics
    wait_until, finish, assume, barrier,
    # Logging
    log,
    # Module decorator
    module,
    # Ports and wires
    Port, WireIn, WireOut,
    # External modules
    ExternalSV, external,
)


def test_intrinsics_dump():
    """Test intrinsic operations IR dump logging."""
    sys_builder = SysBuilder('intrinsics_test')
    
    def test_func():
        class IntrinsicsTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'cond': Port(UInt(1)),
                    'barrier_val': Port(UInt(8))
                })
            
            @module.combinational
            def build(self):
                cond = self.cond.pop()
                barrier_val = self.barrier_val.pop()
                
                # Test intrinsic operations
                wait_result = wait_until(cond)
                finish_result = finish()
                assert_result = assume(cond)
                barrier_result = barrier(barrier_val)
                
                log("Intrinsics test")
        
        IntrinsicsTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Intrinsics Test IR Dump ===")
    print(sys_repr)
    
    # Verify intrinsics appear
    assert "intrinsic.wait_until" in sys_repr
    assert "intrinsic.finish" in sys_repr
    assert "intrinsic.assert" in sys_repr
    assert "intrinsic.barrier" in sys_repr


if __name__ == '__main__':
    test_intrinsics_dump()
    print("\n=== Intrinsics Tests Completed Successfully ===")
