"""Tests for predicate metadata capture on side-effecting IR nodes."""

import os
import sys

sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import Module, SysBuilder, UInt, RegArray, Port, Condition, module  # pylint: disable=import-error


class Callee(Module):
    """Simple callee used to exercise async call metadata."""

    def __init__(self):
        super().__init__(ports={'input_port': Port(UInt(8))})

    @module.combinational
    def build(self):
        # Simple combinational passthrough; no body needed for metadata test.
        pass


def test_side_effects_capture_meta_cond():
    """Ensure ArrayWrite, FIFOPush, FIFOPop, and AsyncCall record predicate metadata."""

    sys = SysBuilder("side_effect_meta_cond")

    with sys:
        class PredicatedOps(Module):  # type: ignore[misc]
            def __init__(self):
                super().__init__(ports={
                    'cond': Port(UInt(1)),
                    'array_idx': Port(UInt(2)),
                    'array_val': Port(UInt(8)),
                    'fifo_in': Port(UInt(8)),
                    'fifo_out': Port(UInt(8)),
                })

            @module.combinational
            def build(self):
                cond = self.cond.pop()
                idx = self.array_idx.pop()
                value = self.array_val.pop()
                array = RegArray(UInt(8), 4, name="meta_store")
                callee = Callee()

                self.saved_cond = cond

                with Condition(cond):
                    write_port = array & self
                    self.write_expr = write_port[idx] <= value

                    self.direct_push = self.fifo_out.push(value)
                    self.pop_expr = self.fifo_in.pop()

                    bound = callee.bind(input_port=value)
                    self.bound_pushes = list(bound.pushes)
                    self.async_expr = bound.async_called(input_port=value)
                    self.async_pushes = list(bound.pushes)

        module_inst = PredicatedOps()
        module_inst.build()

    cond_value = module_inst.saved_cond

    assert module_inst.write_expr.meta_cond is cond_value
    assert module_inst.direct_push.meta_cond is cond_value
    assert module_inst.pop_expr.meta_cond is cond_value
    assert module_inst.async_expr.meta_cond is cond_value

    for push in module_inst.bound_pushes:
        assert push.meta_cond is cond_value

    for push in module_inst.async_pushes:
        assert push.meta_cond is cond_value
