"""Tests for type operation IR nodes: Cast, Concat, Select, Select1Hot, Log."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
    Module, SysBuilder, UInt, Int, Bits, Port, log, module
)
from assassyn.test import dump_ir


def test_cast_concat_select_dump():
    """Test cast, concat, and select IR dump logging."""
    def builder(sys):
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
    
    def checker(sys_repr):
        # Verify operations appear
        assert "bitcast_result =" in sys_repr and "bitcast" in sys_repr
        assert "zext_result =" in sys_repr and "zext" in sys_repr
        assert "sext_result =" in sys_repr and "sext" in sys_repr
        assert "concat_result =" in sys_repr
        assert "select_result =" in sys_repr
    
    dump_ir("cast_concat_select_test", builder, checker)


def test_log_dump():
    """Test log operation IR dump logging."""
    def builder(sys):
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
    
    def checker(sys_repr):
        # Verify log operations appear
        assert "log('Log test message') // meta cond" in sys_repr
        assert "log('Log with value: {}'" in sys_repr
        assert sys_repr.count("// meta cond") >= 2

    dump_ir("log_test", builder, checker)


if __name__ == '__main__':
    test_cast_concat_select_dump()
    test_log_dump()
    print("\n=== Type Operations Tests Completed Successfully ===")
