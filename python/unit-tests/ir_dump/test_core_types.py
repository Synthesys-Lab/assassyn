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
                super().__init__(ports={
                    'uint_val': Port(UInt(8)),
                    'int_val': Port(Int(16)),
                    'bits_val': Port(Bits(4))
                })
            
            @module.combinational
            def build(self):
                # Test different constant types using port pop
                uint_const = self.uint_val.pop()
                int_const = self.int_val.pop()
                bits_const = self.bits_val.pop()
                
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
    
    # Verify actual IR statements appear in the dump
    assert "result1 =" in sys_repr and "uint_const" in sys_repr
    assert "result2 =" in sys_repr and "bits_const" in sys_repr


def test_array_dump():
    """Test array and slicing IR dump logging."""
    sys_builder = SysBuilder('array_test')
    
    def test_func():
        class ArrayTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'index': Port(UInt(2)),
                    'write_val': Port(UInt(8))
                })
            
            @module.combinational
            def build(self):
                # Test RegArray creation
                arr = RegArray(UInt(8), 4, name="test_array")
                
                # Test array read using port pop
                index = self.index.pop()
                val = arr[index]
                
                # Test array write using port pop
                write_val = self.write_val.pop()
                (arr & self)[index] <= write_val
                
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
    assert "val = arr[" in sys_repr
    assert "] <=" in sys_repr


def test_record_dump():
    """Test record type IR dump logging."""
    sys_builder = SysBuilder('record_test')
    
    def test_func():
        class RecordTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'field1_val': Port(UInt(8)),
                    'field2_val': Port(Bits(4)),
                    'field3_val': Port(Int(16))
                })
            
            @module.combinational
            def build(self):
                # Create record type
                record_type = Record(
                    field1=UInt(8),
                    field2=Bits(4),
                    field3=Int(16)
                )
                
                # Create record value using port pop
                field1_val = self.field1_val.pop()
                field2_val = self.field2_val.pop()
                field3_val = self.field3_val.pop()
                
                record_val = record_type.bundle(
                    field1=field1_val,
                    field2=field2_val,
                    field3=field3_val
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
    assert "field_val = bitcast" in sys_repr


if __name__ == '__main__':
    test_const_dump()
    test_array_dump()
    test_record_dump()
    print("\n=== Core Types Tests Completed Successfully ===")
