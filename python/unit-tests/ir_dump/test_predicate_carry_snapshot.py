"""Tests for predicate carry snapshots on general expression nodes."""

import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (  # pylint: disable=import-error
    Condition,
    Module,
    Port,
    SysBuilder,
    UInt,
    module,
)
from assassyn.ir.expr.intrinsic import get_pred  # pylint: disable=import-error


class NestedPredicates(Module):
    """Module that nests three predicate frames to exercise carry propagation."""

    def __init__(self):
        super().__init__(ports={
            'cond_a': Port(UInt(1)),
            'cond_b': Port(UInt(1)),
            'cond_c': Port(UInt(1)),
            'value': Port(UInt(8)),
        })

    @module.combinational
    def build(self):
        cond_a = self.cond_a.pop()
        cond_b = self.cond_b.pop()
        cond_c = self.cond_c.pop()
        value = self.value.pop()

        self.cond_a_value = cond_a
        self.cond_b_value = cond_b
        self.cond_c_value = cond_c

        with Condition(cond_a):
            self.level1_pred = get_pred()
            self.level1_expr = value + UInt(8)(1)

            with Condition(cond_b):
                self.level2_pred = get_pred()
                self.level2_expr = value + UInt(8)(2)

                with Condition(cond_c):
                    self.level3_pred = get_pred()
                    self.level3_expr = value + UInt(8)(3)


def test_predicate_carry_snapshot():
    """Ensure predicate traces capture (cond, carry) ordering with nested conditions."""

    sys_builder = SysBuilder("predicate_carry_snapshot")

    with sys_builder:
        module_inst = NestedPredicates()
        module_inst.build()

    # Level 1: single predicate frame
    level1_trace = module_inst.level1_expr.predicate_trace
    assert len(level1_trace) == 1
    assert level1_trace[0][0] is module_inst.cond_a_value
    assert level1_trace[0][1] is module_inst.level1_pred
    assert module_inst.level1_expr.meta_cond is module_inst.level1_pred

    # Level 2: two predicate frames
    level2_trace = module_inst.level2_expr.predicate_trace
    assert len(level2_trace) == 2
    assert level2_trace[0][0] is module_inst.cond_a_value
    assert level2_trace[0][1] is module_inst.level1_pred
    assert level2_trace[1][0] is module_inst.cond_b_value
    assert level2_trace[1][1] is module_inst.level2_pred
    assert module_inst.level2_expr.meta_cond is module_inst.level2_pred

    # Level 3: three predicate frames with cumulative carry
    level3_trace = module_inst.level3_expr.predicate_trace
    assert len(level3_trace) == 3
    assert level3_trace[0][0] is module_inst.cond_a_value
    assert level3_trace[0][1] is module_inst.level1_pred
    assert level3_trace[1][0] is module_inst.cond_b_value
    assert level3_trace[1][1] is module_inst.level2_pred
    assert level3_trace[2][0] is module_inst.cond_c_value
    assert level3_trace[2][1] is module_inst.level3_pred
    assert module_inst.level3_expr.meta_cond is module_inst.level3_pred

    # Flattened tokens must list cond then carry for each frame
    tokens = module_inst.level3_expr.predicate_tokens
    assert len(tokens) == 6
    assert tokens[0] is module_inst.cond_a_value
    assert tokens[1] is module_inst.level1_pred
    assert tokens[2] is module_inst.cond_b_value
    assert tokens[3] is module_inst.level2_pred
    assert tokens[4] is module_inst.cond_c_value
    assert tokens[5] is module_inst.level3_pred
