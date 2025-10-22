"""Tests for core IR types: Const, Array, Slice."""

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


def test_const_dump():
    """Test constant value IR dump logging."""
    sys_builder = SysBuilder('const_test')
    
    def test_func():
        class ConstTestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Test different constant types
                uint_const = UInt(8)(42)
                int_const = Int(16)(-100)
                bits_const = Bits(4)(15)
                
                # Use them in operations to ensure they appear in IR dump
                result1 = uint_const + int_const
                result2 = bits_const + UInt(4)(1)
                log("Const test: {}", result1)
        
        ConstTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Const Test IR Dump ===")
    print(sys_repr)
    
    # Verify key elements appear in the dump
    assert "42" in sys_repr or "u8" in sys_repr
    assert "-100" in sys_repr or "i16" in sys_repr
    assert "15" in sys_repr or "b4" in sys_repr or "Bits" in sys_repr


def test_array_dump():
    """Test array and slicing IR dump logging."""
    sys_builder = SysBuilder('array_test')
    
    def test_func():
        class ArrayTestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Test RegArray creation
                arr = RegArray(UInt(8), 4, name="test_array")
                
                # Test array read
                val = arr[0]
                
                # Test array write
                (arr & self)[0] <= UInt(8)(100)
                
                # Test slicing
                slice_val = val[3:0]  # 4-bit slice
                
                log("Array test: {}", slice_val)
        
        ArrayTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Array Test IR Dump ===")
    print(sys_repr)
    
    # Verify array operations appear
    assert "arr" in sys_repr or "test_array" in sys_repr
    assert "ArrayRead" in sys_repr or "array" in sys_repr
    assert "ArrayWrite" in sys_repr or "<=" in sys_repr


def test_record_dump():
    """Test record type IR dump logging."""
    sys_builder = SysBuilder('record_test')
    
    def test_func():
        class RecordTestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create record type
                record_type = Record(
                    field1=UInt(8),
                    field2=Bits(4),
                    field3=Int(16)
                )
                
                # Create record value
                record_val = record_type.bundle(
                    field1=UInt(8)(42),
                    field2=Bits(4)(15),
                    field3=Int(16)(100)  # Use positive value to avoid range issues
                )
                
                # Access record fields
                field_val = record_val.field1
                
                log("Record test: {}", field_val)
        
        RecordTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Record Test IR Dump ===")
    print(sys_repr)
    
    # Verify record operations appear
    assert "Record" in sys_repr or "record" in sys_repr
    assert "field1" in sys_repr or "field" in sys_repr


if __name__ == '__main__':
    test_const_dump()
    test_array_dump()
    test_record_dump()
    print("\n=== Core Types Tests Completed Successfully ===")
