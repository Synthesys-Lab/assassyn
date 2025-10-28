"""Unit tests for builder context invariants."""

import pytest

from assassyn.builder import Singleton
from assassyn.frontend import Module, Port, SysBuilder, UInt, module


def test_current_module_requires_active_context():
    """current_module should raise when no module context is active."""
    sys = SysBuilder("builder_current_module_guard")
    with sys:
        builder = Singleton.peek_builder()
        with pytest.raises(RuntimeError):
            _ = builder.current_module


def test_expr_parent_and_body_binding():
    """Expressions emitted through the builder record their parent module."""

    class CaptureModule(Module):
        def __init__(self):
            super().__init__(ports={'lhs': Port(UInt(4)), 'rhs': Port(UInt(4))})
            self.captured_expr = None
            self.captured_body = None
            self.captured_insert_point = None

        @module.combinational
        def build(self):
            builder = Singleton.peek_builder()
            self.captured_body = builder.current_body
            self.captured_insert_point = builder.insert_point

            lhs = self.lhs.pop()
            rhs = self.rhs.pop()
            expr = lhs + rhs

            self.captured_expr = expr

    sys = SysBuilder("builder_parent_assignment")
    with sys:
        CaptureModule().build()

    module_instance = sys.modules[0]
    expr = module_instance.captured_expr

    assert expr is not None
    assert expr.parent is module_instance
    assert module_instance.body is module_instance.captured_body
    assert module_instance.body is module_instance.captured_insert_point
    assert module_instance.body[-1] is expr
