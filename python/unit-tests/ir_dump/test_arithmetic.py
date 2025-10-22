"""Tests for arithmetic IR nodes: BinaryOp, UnaryOp."""

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


def test_binary_ops_dump():
    """Test binary operation IR dump logging."""
    sys_builder = SysBuilder('binary_ops_test')
    
    def test_func():
        class BinaryOpsTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'a': Port(UInt(8)),
                    'b': Port(UInt(8)),
                    'shift_val': Port(UInt(3))
                })
            
            @module.combinational
            def build(self):
                a = self.a.pop()
                b = self.b.pop()
                shift_val = self.shift_val.pop()
                
                # Test all binary operations
                add_result = a + b
                sub_result = a - b
                mul_result = a * b
                and_result = a & b
                or_result = a | b
                xor_result = a ^ b
                lt_result = a < b
                gt_result = a > b
                le_result = a <= b
                ge_result = a >= b
                eq_result = a == b
                ne_result = a != b
                shl_result = a << shift_val
                shr_result = a >> UInt(3)(1)
                
                log("Binary ops test: {}", add_result)
        
        BinaryOpsTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Binary Ops Test IR Dump ===")
    print(sys_repr)
    
    # Verify actual IR statements appear
    assert "add_result =" in sys_repr and "+" in sys_repr
    assert "sub_result =" in sys_repr and "-" in sys_repr
    assert "mul_result =" in sys_repr and "*" in sys_repr
    assert "and_result =" in sys_repr and "&" in sys_repr
    assert "or_result =" in sys_repr and "|" in sys_repr
    assert "xor_result =" in sys_repr and "^" in sys_repr
    assert "lt_result =" in sys_repr and "<" in sys_repr
    assert "gt_result =" in sys_repr and ">" in sys_repr
    assert "le_result =" in sys_repr and "<=" in sys_repr
    assert "ge_result =" in sys_repr and ">=" in sys_repr
    assert "eq_result =" in sys_repr and "==" in sys_repr
    assert "ne_result =" in sys_repr and "!=" in sys_repr
    assert "shl_result =" in sys_repr and "<<" in sys_repr


def test_unary_ops_dump():
    """Test unary operation IR dump logging."""
    sys_builder = SysBuilder('unary_ops_test')
    
    def test_func():
        class UnaryOpsTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'a': Port(UInt(8))
                })
            
            @module.combinational
            def build(self):
                a = self.a.pop()
                
                # Test unary operations
                # Note: __neg__ is not implemented, so we'll test flip only
                flip_result = ~a
                
                # Test manual NEG operation creation using ir_builder
                from assassyn.builder import ir_builder
                from assassyn.ir.expr.arith import UnaryOp
                
                @ir_builder
                def create_neg():
                    return UnaryOp(UnaryOp.NEG, a)
                
                neg_result = create_neg()
                
                log("Unary ops test: {}", neg_result)
        
        UnaryOpsTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Unary Ops Test IR Dump ===")
    print(sys_repr)
    
    # Verify unary operations appear
    assert "flip_result = !" in sys_repr


if __name__ == '__main__':
    test_binary_ops_dump()
    test_unary_ops_dump()
    print("\n=== Arithmetic Tests Completed Successfully ===")
