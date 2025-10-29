"""Type enforcement contract tests for Verilog array expression helpers."""

import pytest

from assassyn.codegen.verilog._expr import array as array_codegen
from assassyn.codegen.verilog.design import CIRCTDumper


HELPERS = [
    array_codegen.codegen_array_read,
    array_codegen.codegen_array_write,
    array_codegen.codegen_fifo_push,
    array_codegen.codegen_fifo_pop,
]


@pytest.mark.parametrize("helper", HELPERS)
def test_helpers_reject_non_circt_dumper(helper):
    """Each helper should raise TypeError if dumper is not a CIRCTDumper."""
    with pytest.raises(TypeError):
        helper(object(), None)  # expr type validation should not be reached


@pytest.mark.parametrize("helper", HELPERS)
def test_helpers_reject_invalid_expr(helper):
    """Each helper should raise TypeError if expr type does not match."""
    dumper = CIRCTDumper()
    with pytest.raises(TypeError):
        helper(dumper, object())
