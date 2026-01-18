"""Test constant folding for select operations.

This test validates that select operations with constant conditions
are properly folded at compile time, avoiding unnecessary code generation.
"""

from assassyn.frontend import *
from assassyn.test import run_test

class TestSelectConstantFolding(Module):
    """Test module for select constant folding."""

    def __init__(self):
        super().__init__(ports={}, no_arbiter=True)

    @module.combinational
    def build(self):
        """Test various constant folding scenarios."""
        # Test case 1: Constant true condition (non-zero single bit)
        true_cond = Bits(1)(1)
        result_true = true_cond.select(Bits(8)(100), Bits(8)(200))
        log('Constant true (Bits(1)(1)): {}', result_true)

        # Test case 2: Constant false condition (zero single bit)
        false_cond = Bits(1)(0)
        result_false = false_cond.select(Bits(8)(100), Bits(8)(200))
        log('Constant false (Bits(1)(0)): {}', result_false)

        # Test case 3: Multi-bit non-zero constant (truthy)
        multi_bit_true = UInt(5)(7)
        result_multi_true = multi_bit_true.select(Bits(8)(42), Bits(8)(99))
        log('Multi-bit non-zero (UInt(5)(7)): {}', result_multi_true)

        # Test case 4: Multi-bit zero constant (falsy)
        multi_bit_false = UInt(5)(0)
        result_multi_false = multi_bit_false.select(Bits(8)(42), Bits(8)(99))
        log('Multi-bit zero (UInt(5)(0)): {}', result_multi_false)

        # Test case 5: Larger constant value
        large_true = UInt(8)(255)
        result_large = large_true.select(Bits(16)(0x1234), Bits(16)(0x5678))
        log('Large constant (UInt(8)(255)): 0x{:x}', result_large)

        # Test case 6: User reproducer from bug report
        # This is the exact case that was failing before the fix
        signal_cnt = Bits(5)(15)
        signal_value = Bits(1)(1)
        log('User reproducer - cnt: {}, signal: {}', signal_cnt, signal_value)


def top():
    """Top-level test function."""
    test = TestSelectConstantFolding()
    test.build()


def check(raw: str):
    """Validate test output."""
    lines = raw.splitlines()
    for line in lines:
        if 'Constant true (Bits(1)(1)):' in line:
            result = int(line.split()[-1])
            assert result == 100, f"Expected 100 for true condition, got {result}"
        elif 'Constant false (Bits(1)(0)):' in line:
            result = int(line.split()[-1])
            assert result == 200, f"Expected 200 for false condition, got {result}"
        elif 'Multi-bit non-zero (UInt(5)(7)):' in line:
            result = int(line.split()[-1])
            assert result == 42, f"Expected 42 for multi-bit true, got {result}"
        elif 'Multi-bit zero (UInt(5)(0)):' in line:
            result = int(line.split()[-1])
            assert result == 99, f"Expected 99 for multi-bit false, got {result}"
        elif 'Large constant (UInt(8)(255)):' in line:
            result_hex = line.split()[-1]
            result = int(result_hex, 16)
            assert result == 0x1234, f"Expected 0x1234, got {result_hex}"
        elif 'User reproducer - cnt:' in line:
            parts = line.split()
            cnt = int(parts[parts.index('cnt:') + 1].rstrip(','))
            signal = int(parts[parts.index('signal:') + 1])
            assert cnt == 15, f"Expected cnt=15, got {cnt}"
            assert signal == 1, f"Expected signal=1, got {signal}"


def test_select_constant_folding():
    """Run the select constant folding test."""
    run_test('select_constant_folding', top, check)


if __name__ == '__main__':
    test_select_constant_folding()
