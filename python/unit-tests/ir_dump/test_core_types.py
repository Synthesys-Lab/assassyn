"""Tests for core IR types: Const, Array, Slice."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
    Module, SysBuilder, UInt, Int, Bits, Record,
    RegArray, Port, log, module
)
from assassyn.test import dump_ir


def test_const_dump():
    """Test constant value IR dump logging."""
    def builder(sys):
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
        
        # Create and build module
        module_instance = ConstTestModule()
        module_instance.build()
    
    def checker(sys_repr):
        # Verify actual IR statements appear in the dump
        assert "result1 =" in sys_repr and "uint_const" in sys_repr
        assert "result2 =" in sys_repr and "bits_const" in sys_repr
    
    dump_ir("const_test", builder, checker)


def test_array_dump():
    """Test array and slicing IR dump logging."""
    def builder(sys):
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
    
    def checker(sys_repr):
        # Verify array operations appear
        assert "val = arr[" in sys_repr
        assert "] <=" in sys_repr
    
    dump_ir("array_test", builder, checker)


def test_record_dump():
    """Test record type IR dump logging."""
    def builder(sys):
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
    
    def checker(sys_repr):
        # Verify record operations appear
        assert "field_val = bitcast" in sys_repr
    
    dump_ir("record_test", builder, checker)


if __name__ == '__main__':
    test_const_dump()
    test_array_dump()
    test_record_dump()
    print("\n=== Core Types Tests Completed Successfully ===")
