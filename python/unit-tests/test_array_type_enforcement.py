"""Test array write type enforcement.

This module tests that array writes enforce strict type checking between
the written value and the array's scalar_ty, including proper RecordValue handling.
"""

import pytest
from assassyn.ir.array import RegArray
from assassyn.ir.dtype import UInt, Bits, Record, RecordValue
from assassyn.ir.const import Const
from assassyn.ir.module import Module, module
from assassyn.builder import SysBuilder


def test_array_write_correct_type():
    """Test that writing a value with correct type succeeds."""
    sys = SysBuilder("test_array_write_correct_type")
    with sys:
        # Create a module context
        class TestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create array of UInt(8)
                array = RegArray(UInt(8), 4, name="test_array")
                
                # Write correct type should succeed
                value = Const(UInt(8), 42)
                array[0] = value
                
                # Verify the write was created
                assert len(array.users) > 0
        
        module_instance = TestModule()
        module_instance.build()


def test_array_write_incorrect_type():
    """Test that writing a value with incorrect type raises TypeError."""
    sys = SysBuilder("test_array_write_incorrect_type")
    with sys:
        # Create a module context
        class TestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create array of UInt(8)
                array = RegArray(UInt(8), 4, name="test_array")
                
                # Write incorrect type should raise TypeError
                wrong_value = Const(UInt(16), 42)
                with pytest.raises(TypeError) as exc_info:
                    array[0] = wrong_value
                
                error_msg = str(exc_info.value)
                assert "Type mismatch" in error_msg
                assert "u8" in error_msg or "UInt(8)" in error_msg
                assert "u16" in error_msg or "UInt(16)" in error_msg
        
        module_instance = TestModule()
        module_instance.build()


def test_array_write_bits_mismatch():
    """Test that writing Bits with wrong width raises TypeError."""
    sys = SysBuilder("test_array_write_bits_mismatch")
    with sys:
        # Create a module context
        class TestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create array of Bits(8)
                array = RegArray(Bits(8), 4, name="test_array")
                
                # Write Bits with different width should raise TypeError
                wrong_value = Const(Bits(16), 42)
                with pytest.raises(TypeError) as exc_info:
                    array[0] = wrong_value
                
                assert "Type mismatch" in str(exc_info.value)
        
        module_instance = TestModule()
        module_instance.build()


def test_array_write_record_correct_type():
    """Test that writing RecordValue with matching type succeeds and unwraps."""
    sys = SysBuilder("test_array_write_record_correct_type")
    with sys:
        # Create a module context
        class TestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create a Record type
                rec_type = Record(a=UInt(8), b=UInt(16))
                
                # Create array of this Record type
                array = RegArray(rec_type, 4, name="test_array")
                
                # Create RecordValue with matching type
                rec_value = RecordValue(rec_type, a=Const(UInt(8), 1), b=Const(UInt(16), 2))
                
                # Write should succeed and unwrap the RecordValue
                array[0] = rec_value
                
                # Verify the write was created
                assert len(array.users) > 0
        
        module_instance = TestModule()
        module_instance.build()


def test_array_write_record_incorrect_type():
    """Test that writing RecordValue with mismatching type raises TypeError."""
    sys = SysBuilder("test_array_write_record_incorrect_type")
    with sys:
        # Create a module context
        class TestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create Record types
                rec_type1 = Record(a=UInt(8), b=UInt(16))
                rec_type2 = Record(x=UInt(8), y=UInt(16))  # Different field names
                
                # Create array of rec_type1
                array = RegArray(rec_type1, 4, name="test_array")
                
                # Create RecordValue with different type
                rec_value = RecordValue(rec_type2, x=Const(UInt(8), 1), y=Const(UInt(16), 2))
                
                # Write should raise TypeError
                with pytest.raises(TypeError) as exc_info:
                    array[0] = rec_value
                
                assert "Type mismatch" in str(exc_info.value)
        
        module_instance = TestModule()
        module_instance.build()


def test_multiport_write_correct_type():
    """Test multi-port write with (array & module)[idx] <= value syntax."""
    sys = SysBuilder("test_multiport_write_correct_type")
    with sys:
        # Create two modules
        class Module1(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create array of UInt(8)
                array = RegArray(UInt(8), 4, name="test_array")
                
                # Multi-port write with correct type should succeed
                value1 = Const(UInt(8), 10)
                (array & self)[0] <= value1
        
        class Module2(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create array of UInt(8)
                array = RegArray(UInt(8), 4, name="test_array")
                
                # Another port write with correct type
                value2 = Const(UInt(8), 20)
                (array & self)[1] <= value2
        
        module1 = Module1()
        module2 = Module2()
        module1.build()
        module2.build()


def test_multiport_write_incorrect_type():
    """Test multi-port write rejects incorrect type."""
    sys = SysBuilder("test_multiport_write_incorrect_type")
    with sys:
        # Create a module
        class TestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create array of UInt(8)
                array = RegArray(UInt(8), 4, name="test_array")
                
                # Multi-port write with incorrect type should raise TypeError
                wrong_value = Const(UInt(16), 42)
                with pytest.raises(TypeError) as exc_info:
                    (array & self)[0] <= wrong_value
                
                error_msg = str(exc_info.value)
                assert "Type mismatch" in error_msg
                assert "u8" in error_msg or "UInt(8)" in error_msg
                assert "u16" in error_msg or "UInt(16)" in error_msg
        
        module_instance = TestModule()
        module_instance.build()


def test_multiport_write_record_unwrapping():
    """Test multi-port write properly unwraps RecordValue."""
    sys = SysBuilder("test_multiport_write_record_unwrapping")
    with sys:
        # Create a module
        class TestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Create a Record type
                rec_type = Record(a=UInt(8), b=UInt(16))
                
                # Create array of this Record type
                array = RegArray(rec_type, 4, name="test_array")
                
                # Create RecordValue with matching type
                rec_value = RecordValue(rec_type, a=Const(UInt(8), 1), b=Const(UInt(16), 2))
                
                # Multi-port write should succeed and unwrap
                (array & self)[0] <= rec_value
                
                # Verify the write was created
                assert len(array.users) > 0
        
        module_instance = TestModule()
        module_instance.build()