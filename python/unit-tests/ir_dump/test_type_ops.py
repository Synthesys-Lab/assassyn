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
                super().__init__(ports={
                    'a': Port(UInt(8)),
                    'b': Port(UInt(8)),
                    'c': Port(UInt(4)),
                    'cond': Port(UInt(1))
                })
            
            @module.combinational
            def build(self):
                a = self.a.pop()
                b = self.b.pop()
                c = self.c.pop()
                cond = self.cond.pop()
                
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
    assert "bitcast_result =" in sys_repr and "bitcast" in sys_repr
    assert "zext_result =" in sys_repr and "zext" in sys_repr
    assert "sext_result =" in sys_repr and "sext" in sys_repr
    assert "concat_result =" in sys_repr
    assert "select_result =" in sys_repr


def test_log_dump():
    """Test log operation IR dump logging."""
    sys_builder = SysBuilder('log_test')
    
    def test_func():
        class LogTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'log_val': Port(UInt(8))
                })
            
            @module.combinational
            def build(self):
                # Test log operation
                log_val = self.log_val.pop()
                log("Log test message")
                log("Log with value: {}", log_val)
        
        LogTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Log Test IR Dump ===")
    print(sys_repr)
    
    # Verify log operations appear
    assert "log('Log test message'" in sys_repr


if __name__ == '__main__':
    test_cast_concat_select_dump()
    test_log_dump()
    print("\n=== Type Operations Tests Completed Successfully ===")
