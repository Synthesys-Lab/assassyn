"""Tests for block IR nodes: Block, CondBlock, CycledBlock."""

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


def test_block_dump():
    """Test block IR dump logging."""
    sys_builder = SysBuilder('block_test')
    
    def test_func():
        class BlockTestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                cond = UInt(1)(1)
                
                # Test conditional block
                with Condition(cond):
                    log("In conditional block")
                
                # Test cycled block
                with Cycle(5):
                    log("In cycle block")
        
        BlockTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Block Test IR Dump ===")
    print(sys_repr)
    
    # Verify block structures appear
    assert "when" in sys_repr or "Condition" in sys_repr
    assert "cycle" in sys_repr or "Cycle" in sys_repr


if __name__ == '__main__':
    test_block_dump()
    print("\n=== Block Tests Completed Successfully ===")
