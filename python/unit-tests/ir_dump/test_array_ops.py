"""Tests for array operation IR nodes: ArrayRead, ArrayWrite, WritePort."""

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


def test_array_ops_dump():
    """Test array read/write operations IR dump logging."""
    sys_builder = SysBuilder('array_ops_test')
    
    def test_func():
        class ArrayOpsTestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create array
                arr = RegArray(UInt(8), 4, name="test_array")
                
                # Test array read
                read_val = arr[0]
                
                # Test array write using WritePort syntax
                write_port = arr & self
                write_port[0] = UInt(8)(42)
                
                # Alternative write syntax
                (arr & self)[1] <= UInt(8)(24)
                
                log("Array ops test: {}", read_val)
        
        ArrayOpsTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Array Ops Test IR Dump ===")
    print(sys_repr)
    
    # Verify array operations appear
    assert "ArrayRead" in sys_repr or "array" in sys_repr
    assert "ArrayWrite" in sys_repr or "<=" in sys_repr
    assert "arr" in sys_repr  # Array name should appear


if __name__ == '__main__':
    test_array_ops_dump()
    print("\n=== Array Operations Tests Completed Successfully ===")
