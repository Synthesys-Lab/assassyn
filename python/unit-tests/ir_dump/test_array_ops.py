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
                super().__init__(ports={
                    'read_index': Port(UInt(2)),
                    'write_index': Port(UInt(2)),
                    'write_val': Port(UInt(8))
                })
            
            @module.combinational
            def build(self):
                # Create array
                arr = RegArray(UInt(8), 4, name="test_array")
                
                # Test array read using port pop
                read_index = self.read_index.pop()
                read_val = arr[read_index]
                
                # Test array write using WritePort syntax with port pop
                write_index = self.write_index.pop()
                write_val = self.write_val.pop()
                write_port = arr & self
                write_port[write_index] = write_val
                
                # Alternative write syntax
                (arr & self)[write_index] <= write_val
                
                log("Array ops test: {}", read_val)
        
        ArrayOpsTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Array Ops Test IR Dump ===")
    print(sys_repr)
    
    # Verify array operations appear
    assert "read_val = arr[" in sys_repr
    assert "] <=" in sys_repr


if __name__ == '__main__':
    test_array_ops_dump()
    print("\n=== Array Operations Tests Completed Successfully ===")
