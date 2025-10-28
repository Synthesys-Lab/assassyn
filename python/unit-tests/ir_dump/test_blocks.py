"""Tests for predicate intrinsics and Cycle-as-conditional sugar."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import Module, SysBuilder, UInt, Condition, Cycle, Port, log, module
from assassyn.ir.const import Const
from assassyn.ir.expr import Log
from assassyn.test import dump_ir


def test_block_dump():
    """Test block IR dump logging."""
    def builder(sys):
        class BlockTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'cond': Port(UInt(1))
                })
            
            @module.combinational
            def build(self):
                cond = self.cond.pop()
                
                # Test conditional block
                with Condition(cond):
                    log("In conditional block")
                
                # Test cycled block
                with Cycle(5):
                    log("In cycle block")
        
        BlockTestModule().build()
    
    def checker(sys_repr):
        # Verify predicate-based conditional emission appears in IR dump as braces with comments
        assert "// PUSH_CONDITION" in sys_repr
        assert "} // POP_CONDITION" in sys_repr
        assert "if " in sys_repr
        assert "In conditional block" in sys_repr
        assert "In cycle block" in sys_repr
    
    dump_ir("block_test", builder, checker)


def test_log_meta_cond_metadata():
    """Ensure log nodes record their predicate metadata."""

    sys = SysBuilder("log_meta_cond")

    with sys:
        class LogPredicateModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'cond': Port(UInt(1))
                })

            @module.combinational
            def build(self):
                cond = self.cond.pop()
                self.saved_cond = cond

                log("Outer log")

                with Condition(cond):
                    log("Inner log", cond)

        module_inst = LogPredicateModule()
        module_inst.build()

    top_module = sys.modules[0]

    def _gather_logs(nodes):
        found = []
        for elem in nodes:
            if isinstance(elem, Log):
                found.append(elem)
            body = getattr(elem, 'body', None)
            if isinstance(body, list):
                found.extend(_gather_logs(body))
        return found

    logs = _gather_logs(top_module.body)

    assert logs, "Expected at least one log node"
    outer_log = logs[0]
    assert isinstance(outer_log.meta_cond, Const)
    assert outer_log.meta_cond.value == 1
    assert outer_log.fmt == "Outer log"
    assert outer_log.values == ()

    assert len(logs) >= 2, "Expected inner log to be present"
    inner_log = logs[1]
    assert inner_log.meta_cond is module_inst.saved_cond
    assert inner_log.fmt == "Inner log"
    assert inner_log.values == (module_inst.saved_cond,)


if __name__ == '__main__':
    test_block_dump()
    print("\n=== Block Tests Completed Successfully ===")
