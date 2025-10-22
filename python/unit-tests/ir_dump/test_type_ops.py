"""Tests for type operation IR nodes: Cast, Concat, Select, Select1Hot, Log."""

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


def test_cast_concat_select_dump():
    """Test cast, concat, and select IR dump logging."""
    sys_builder = SysBuilder('cast_concat_select_test')
    
    def test_func():
        class CastConcatSelectTestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                a = UInt(8)(10)
                b = UInt(8)(5)  # Make same width as a
                c = UInt(4)(3)  # Different width for casting
                cond = UInt(1)(1)
                
                # Test casting operations
                bitcast_result = a.bitcast(Bits(8))
                zext_result = c.zext(UInt(8))
                sext_result = c.sext(Int(8))
                
                # Test concatenation
                concat_result = a.concat(c)
                
                # Test select operations (same dtype)
                select_result = cond.select(a, b)
                
                # Test case operation
                case_result = cond.case({
                    UInt(1)(0): a,
                    UInt(1)(1): b,
                    None: UInt(8)(0)  # default
                })
                
                log("Cast/Concat/Select test: {}", select_result)
        
        CastConcatSelectTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Cast/Concat/Select Test IR Dump ===")
    print(sys_repr)
    
    # Verify operations appear
    assert "bitcast" in sys_repr or "BITCAST" in sys_repr
    assert "zext" in sys_repr or "ZEXT" in sys_repr
    assert "sext" in sys_repr or "SEXT" in sys_repr
    assert "concat" in sys_repr or "CONCAT" in sys_repr
    assert "?" in sys_repr or "SELECT" in sys_repr


def test_log_dump():
    """Test log operation IR dump logging."""
    sys_builder = SysBuilder('log_test')
    
    def test_func():
        class LogTestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                # Test log operation
                log("Log test message")
                log("Log with value: {}", UInt(8)(42))
        
        LogTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Log Test IR Dump ===")
    print(sys_repr)
    
    # Verify log operations appear
    assert "log" in sys_repr or "Log" in sys_repr
    assert "Log test message" in sys_repr


if __name__ == '__main__':
    test_cast_concat_select_dump()
    test_log_dump()
    print("\n=== Type Operations Tests Completed Successfully ===")
