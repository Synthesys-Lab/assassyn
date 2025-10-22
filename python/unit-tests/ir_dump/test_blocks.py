"""Tests for block IR nodes: Block, CondBlock, CycledBlock."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
    Module, SysBuilder, UInt, Condition, Cycle,
    Port, log, module
)
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
        # Verify block structures appear
        assert "when" in sys_repr
    
    dump_ir("block_test", builder, checker)


if __name__ == '__main__':
    test_block_dump()
    print("\n=== Block Tests Completed Successfully ===")
