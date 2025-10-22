"""Tests for communicative operation helpers: add, mul, and_, or_, xor, concat."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
    Module, SysBuilder, UInt, Port, log, module
)
from assassyn.test import dump_ir


def test_comm_ops_dump():
    """Test communicative operations IR dump logging."""
    def builder(sys):
        class CommOpsTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'a': Port(UInt(8)),
                    'b': Port(UInt(8)),
                    'c': Port(UInt(8))
                })
            
            @module.combinational
            def build(self):
                from assassyn.ir.expr.comm import add, mul, and_, or_, xor, concat
                
                a = self.a.pop()
                b = self.b.pop()
                c = self.c.pop()
                
                # Test communicative operations
                add_result = add(a, b, c)
                mul_result = mul(a, b)
                and_result = and_(a, b, c)
                or_result = or_(a, b)
                xor_result = xor(a, b, c)
                concat_result = concat(a, b, c)
                
                log("Comm ops test: {}", concat_result)
        
        CommOpsTestModule().build()
    
    def checker(sys_repr):
        # Verify communicative operations appear
        assert "add_result =" in sys_repr and "+" in sys_repr
        assert "mul_result =" in sys_repr and "*" in sys_repr
        assert "and_result =" in sys_repr and "&" in sys_repr
        assert "or_result =" in sys_repr and "|" in sys_repr
        assert "xor_result =" in sys_repr and "^" in sys_repr
        assert "concat_result =" in sys_repr
    
    dump_ir("comm_ops_test", builder, checker)


if __name__ == '__main__':
    test_comm_ops_dump()
    print("\n=== Communicative Operations Tests Completed Successfully ===")
