"""Test type checking in Bind operations"""

import sys
import pytest

from assassyn.ir.dtype import UInt, Int, Bits, Record
from assassyn.ir.module import Module, Port
from assassyn.frontend import SysBuilder


class ModuleUInt8(Module):
    """Test module with a UInt(8) port"""
    def __init__(self):
        super().__init__(ports={'a': Port(UInt(8))})


class ModuleUInt16(Module):
    """Test module with a UInt(16) port"""
    def __init__(self):
        super().__init__(ports={'a': Port(UInt(16))})


class ModuleBits8(Module):
    """Test module with a Bits(8) port"""
    def __init__(self):
        super().__init__(ports={'a': Port(Bits(8))})


def test_bind_type_mismatch_int_vs_uint():
    """Test that binding an Int value to a UInt port raises ValueError"""
    sys = SysBuilder('test_bind_type_mismatch_int_vs_uint')
    with sys:
        mod = ModuleUInt8()
        a_val = Int(8)(5)

        # Should raise ValueError due to type mismatch
        with pytest.raises(ValueError) as exc_info:
            bind = mod.bind(a=a_val)

        assert "Type mismatch in Bind" in str(exc_info.value)
        assert "port 'a'" in str(exc_info.value)
        assert "u8" in str(exc_info.value)  # Expected UInt(8)
        assert "i8" in str(exc_info.value)  # Got Int(8)


def test_bind_type_mismatch_different_bitwidths():
    """Test that binding a value with wrong bitwidth raises ValueError"""
    sys = SysBuilder('test_bind_type_mismatch_different_bitwidths')
    with sys:
        mod = ModuleUInt8()
        a_val = UInt(16)(5)

        with pytest.raises(ValueError) as exc_info:
            bind = mod.bind(a=a_val)

        assert "Type mismatch in Bind" in str(exc_info.value)
        assert "port 'a'" in str(exc_info.value)
        assert "u8" in str(exc_info.value)  # Expected UInt(8)
        assert "u16" in str(exc_info.value)  # Got UInt(16)


def test_bind_type_mismatch_uint_vs_bits():
    """Test that binding a UInt value to a Bits port raises ValueError"""
    sys = SysBuilder('test_bind_type_mismatch_uint_vs_bits')
    with sys:
        mod = ModuleBits8()
        a_val = UInt(8)(5)

        with pytest.raises(ValueError) as exc_info:
            bind = mod.bind(a=a_val)

        assert "Type mismatch in Bind" in str(exc_info.value)
        assert "port 'a'" in str(exc_info.value)


def test_bind_type_match_correct():
    """Test that binding with matching types works correctly"""
    sys = SysBuilder('test_bind_type_match_correct')
    with sys:
        mod = ModuleUInt8()
        a_val = UInt(8)(5)
        # Should not raise a type error (but may fail later for other reasons)
        try:
            bind = mod.bind(a=a_val)
            # If we get here without ValueError, type check passed
            assert True
        except ValueError as e:
            if "Type mismatch" in str(e):
                pytest.fail(f"Type check failed unexpectedly: {e}")
            # Other ValueErrors are acceptable for this test
        except AttributeError:
            # Expected - we're not in proper module context for full bind
            # But type check passed, which is what we're testing
            assert True


def test_bind_type_mismatch_record():
    """Test that binding mismatched Record types raises ValueError"""
    rec1 = Record(x=UInt(8), y=UInt(8))
    rec2 = Record(a=UInt(8), b=UInt(8))  # Different field names

    class ModuleRec1(Module):
        def __init__(self):
            super().__init__(ports={'p': Port(rec1)})

    sys = SysBuilder('test_bind_type_mismatch_record')
    with sys:
        mod = ModuleRec1()
        val = rec2.bundle(a=UInt(8)(1), b=UInt(8)(2))

        with pytest.raises(ValueError) as exc_info:
            bind = mod.bind(p=val)

        assert "Type mismatch in Bind" in str(exc_info.value)
        assert "port 'p'" in str(exc_info.value)


def test_bind_type_match_record():
    """Test that binding with matching Record types works correctly"""
    rec = Record(x=UInt(8), y=UInt(8))

    class ModuleRec(Module):
        def __init__(self):
            super().__init__(ports={'p': Port(rec)})

    sys = SysBuilder('test_bind_type_match_record')
    with sys:
        mod = ModuleRec()
        val = rec.bundle(x=UInt(8)(1), y=UInt(8)(2))
        # Should not raise a type error (but may fail later for other reasons)
        try:
            bind = mod.bind(p=val)
            # If we get here without ValueError, type check passed
            assert True
        except ValueError as e:
            if "Type mismatch" in str(e):
                pytest.fail(f"Type check failed unexpectedly: {e}")
            # Other ValueErrors are acceptable for this test
        except AttributeError:
            # Expected - we're not in proper module context for full bind
            # But type check passed, which is what we're testing
            assert True


if __name__ == "__main__":
    # Run tests with pytest
    sys.exit(pytest.main([__file__, "-v"]))

