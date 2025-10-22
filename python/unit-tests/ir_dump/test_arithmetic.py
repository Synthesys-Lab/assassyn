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
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                a = UInt(8)(10)
                b = UInt(8)(5)
                
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
                shl_result = a << UInt(3)(2)
                shr_result = a >> UInt(3)(1)
                
                log("Binary ops test: {}", add_result)
        
        BinaryOpsTestModule().build()
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Binary Ops Test IR Dump ===")
    print(sys_repr)
    
    # Verify operators appear
    assert "+" in sys_repr or "ADD" in sys_repr
    assert "-" in sys_repr or "SUB" in sys_repr
    assert "*" in sys_repr or "MUL" in sys_repr
    assert "&" in sys_repr or "BITWISE_AND" in sys_repr
    assert "|" in sys_repr or "BITWISE_OR" in sys_repr
    assert "^" in sys_repr or "BITWISE_XOR" in sys_repr
    assert "<" in sys_repr or "ILT" in sys_repr
    assert ">" in sys_repr or "IGT" in sys_repr
    assert "<=" in sys_repr or "ILE" in sys_repr
    assert ">=" in sys_repr or "IGE" in sys_repr
    assert "==" in sys_repr or "EQ" in sys_repr
    assert "!=" in sys_repr or "NEQ" in sys_repr
    assert "<<" in sys_repr or "SHL" in sys_repr
    assert ">>" in sys_repr or "SHR" in sys_repr


def test_unary_ops_dump():
    """Test unary operation IR dump logging."""
    sys_builder = SysBuilder('unary_ops_test')
    
    def test_func():
        class UnaryOpsTestModule(Module):
            def __init__(self):
                super().__init__(ports={})
            
            @module.combinational
            def build(self):
                a = UInt(8)(10)
                
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
    
    # Verify unary operators appear
    assert "-" in sys_repr or "NEG" in sys_repr
    assert "!" in sys_repr or "FLIP" in sys_repr


if __name__ == '__main__':
    test_binary_ops_dump()
    test_unary_ops_dump()
    print("\n=== Arithmetic Tests Completed Successfully ===")
