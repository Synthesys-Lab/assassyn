"""Simulator codegen tests for wide integer and BigInt lowering."""

from assassyn.codegen.simulator import utils as sim_utils
from assassyn.codegen.simulator._expr import codegen_concat, codegen_slice
from assassyn.codegen.simulator._expr.arith import codegen_binary_op
from assassyn.ir.array import Slice
from assassyn.ir.dtype import Bits, Int, UInt
from assassyn.ir.expr import BinaryOp, Concat


def test_wide_unsigned_immediate_uses_decimal_helper():
    """Wide unsigned immediates must not be truncated through a u64 cast."""

    value = (1 << 80) + 0x55

    assert sim_utils.int_imm_dumper_impl(Bits(96), value) == (
        f'biguint_from_decimal("{value}")'
    )


def test_wide_signed_immediate_uses_decimal_helper():
    """Wide signed immediates must not be truncated through an i64 cast."""

    value = -(1 << 70) + 13

    assert sim_utils.int_imm_dumper_impl(Int(96), value) == (
        f'bigint_from_decimal("{value}")'
    )


def test_wide_slice_uses_runtime_mask_helper():
    """Wide slices should compute masks numerically instead of parsing strings."""

    node = Slice(Bits(128)((1 << 100) + 0x1234), 65, 127)
    generated = codegen_slice(node, module_ctx=None)

    assert "BigUint::parse_bytes" not in generated
    assert "biguint_mask(63)" in generated


def test_small_concat_uses_u64_fast_path():
    """Concats that fit in a scalar should avoid BigUint conversion."""

    node = Concat(UInt(16)(0x12), UInt(8)(0x34))
    generated = codegen_concat(node, module_ctx=None)

    assert "BigUint" not in generated
    assert "ValueCastTo::<u64>" in generated
    assert "<< 8" in generated
    assert "ValueCastTo::<u32>::cast(&c)" in generated


def test_signed_shift_uses_rounded_rust_scalar_type():
    """Signed right shifts on i24 should lower through i32, not invalid i24."""

    node = BinaryOp(BinaryOp.SHR, Int(24)(-4), UInt(5)(1))
    generated = codegen_binary_op(node, module_ctx=None)

    assert "ValueCastTo::<i24>" not in generated
    assert generated.count("ValueCastTo::<i32>") == 2
