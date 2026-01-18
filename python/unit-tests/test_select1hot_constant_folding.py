"""Test constant folding for select1hot operations.

This test validates that select1hot operations with all constant operands
are properly folded at compile time, avoiding unnecessary code generation.
"""

from assassyn.frontend import *
from assassyn.test import run_test

class TestConstantFolding(Module):
    """Test module for select1hot constant folding."""

    def __init__(self):
        super().__init__(ports={}, no_arbiter=True)

    @module.combinational
    def build(self):
        """Test various constant folding scenarios."""
        # Test case from the bug report - bit 10 set, selecting from 32 values
        select = [Bits(5)(i) for i in range(32)]
        one_hot = Bits(32)(1 << 10)
        select_one_hot = one_hot.select1hot(*select)
        log('Select one hot bit 10: {}', select_one_hot)

        # Test bit 0 selection
        one_hot_0 = Bits(4)(1 << 0)
        result_0 = one_hot_0.select1hot(Bits(8)(100), Bits(8)(200), Bits(8)(50), Bits(8)(150))
        log('Test bit 0: {}', result_0)

        # Test bit 3 selection (last bit in 4-bit selector)
        one_hot_3 = Bits(4)(1 << 3)
        result_3 = one_hot_3.select1hot(Bits(8)(100), Bits(8)(200), Bits(8)(50), Bits(8)(150))
        log('Test bit 3: {}', result_3)

        # Test bit 1 selection
        one_hot_1 = Bits(4)(1 << 1)
        result_1 = one_hot_1.select1hot(Bits(8)(10), Bits(8)(20), Bits(8)(30), Bits(8)(40))
        log('Test bit 1: {}', result_1)

        # Test with different data types
        one_hot_2 = Bits(8)(1 << 2)
        result_2 = one_hot_2.select1hot(
            Bits(16)(0x1234),
            Bits(16)(0x5678),
            Bits(16)(0xABCD),
            Bits(16)(0xEF01),
            Bits(16)(0x2345),
            Bits(16)(0x6789),
            Bits(16)(0xBCDE),
            Bits(16)(0xF012)
        )
        log('Test bit 2 with 16-bit values: 0x{:x}', result_2)


def top():
    """Top-level test function."""
    test = TestConstantFolding()
    test.build()


def check(raw: str):
    """Validate test output."""
    lines = raw.splitlines()
    for line in lines:
        if 'Select one hot bit 10:' in line:
            result = int(line.split()[-1])
            assert result == 10, f"Expected 10, got {result}"
        elif 'Test bit 0:' in line:
            result = int(line.split()[-1])
            assert result == 100, f"Expected 100, got {result}"
        elif 'Test bit 3:' in line:
            result = int(line.split()[-1])
            assert result == 150, f"Expected 150, got {result}"
        elif 'Test bit 1:' in line:
            result = int(line.split()[-1])
            assert result == 20, f"Expected 20, got {result}"
        elif 'Test bit 2 with 16-bit values:' in line:
            result_hex = line.split()[-1]
            result = int(result_hex, 16)
            assert result == 0xABCD, f"Expected 0xABCD, got {result_hex}"


def test_select1hot_constant_folding():
    """Run the select1hot constant folding test."""
    run_test('select1hot_constant_folding', top, check)


if __name__ == '__main__':
    test_select1hot_constant_folding()
