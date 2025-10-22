"""Tests for array operation IR nodes: ArrayRead, ArrayWrite, WritePort."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
    Module, SysBuilder, UInt, RegArray, Port, log, module
)
from assassyn.test import dump_ir


def test_array_ops_dump():
    """Test array read/write operations IR dump logging."""
    def builder(sys):
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
    
    def checker(sys_repr):
        # Verify new array representation format
        assert "arr = [u8; 4];" in sys_repr
        assert "|- Read  by:" in sys_repr
        assert "|- Write by:" in sys_repr
        assert "`- Write by:" in sys_repr
        assert "in ArrayOpsTestModuleInstance" in sys_repr
        assert "read_val = arr[read_index_1]" in sys_repr
        assert "arr[write_index_1] <= write_val_1" in sys_repr
    
    dump_ir("array_ops_test", builder, checker)


if __name__ == '__main__':
    test_array_ops_dump()
    print("\n=== Array Operations Tests Completed Successfully ===")
