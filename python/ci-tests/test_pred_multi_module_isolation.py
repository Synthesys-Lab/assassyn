from assassyn.frontend import *
from assassyn.test import run_test


class A(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        # Intentionally push without pop to expose cross-module leakage if not isolated
        from assassyn.ir.expr.intrinsic import current_cycle
        cond = current_cycle() < UInt(64)(3)
        push_condition(cond)
        log('A active at cycle <3: {}', current_cycle())
        pop_condition()


class B(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        # B should be independent; if A leaked its predicate, B output will be gated by A's cond
        from assassyn.ir.expr.intrinsic import current_cycle
        with Condition(current_cycle() < UInt(64)(5)):
            log('B active (<5): {}', current_cycle())


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, a: A, b: B):
        from assassyn.ir.expr.intrinsic import current_cycle
        from assassyn.ir.dtype import UInt
        a.async_called()
        b.async_called()
        with Condition(current_cycle() == UInt(64)(8)):
            a.build()
            b.build()
            finish()


def check(raw):
    # Expect B to log exactly for cycles 0,1,2,3,4 (five lines)
    b_lines = [ln for ln in raw.split('\n') if 'B active' in ln]
    assert len(b_lines) == 3, f'expected 3 B lines, got {len(b_lines)}. Raw:\n{raw}'


def test_pred_multi_module_isolation():
    def top():
        a = A()
        b = B()
        tb = Driver()
        tb.build(a, b)

    # Simulator only; Verilog enablement not required for this isolation test
    run_test('pred_multi_module_isolation', top, check, sim_threshold=8)


if __name__ == '__main__':
    test_pred_multi_module_isolation()