"""Tests for call operation IR nodes: Bind, AsyncCall, FIFOPush, FIFOPop."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
    Module, SysBuilder, UInt, Port, log, module
)
from assassyn.test import dump_ir


# Create a simple callee module for call ops test
class CalleeModule(Module):
    def __init__(self):
        super().__init__(ports={
            'input_port': Port(UInt(8)),
            'output_port': Port(UInt(8))
        })
    
    @module.combinational
    def build(self):
        # Simple passthrough
        self.output_port <= self.input_port


def test_call_ops_dump():
    """Test call operations IR dump logging."""
    def builder(sys):
        class CallOpsTestModule(Module):
            def __init__(self):
                super().__init__(ports={
                    'bind_arg': Port(UInt(8)),
                    'async_arg': Port(UInt(8))
                })
            
            @module.combinational
            def build(self):
                callee = CalleeModule()
                
                # Test bind operation using port pop
                bind_arg = self.bind_arg.pop()
                bind_result = callee.bind(input_port=bind_arg)
                
                # Test async call using port pop
                async_arg = self.async_arg.pop()
                async_result = bind_result.async_called(input_port=async_arg)
                
                log("Call ops test: {}", async_result)
        
        CallOpsTestModule().build()
    
    def checker(sys_repr):
        # Verify call operations appear
        assert "bind_result =" in sys_repr and "bind" in sys_repr
        assert "async_call" in sys_repr
    
    dump_ir("call_ops_test", builder, checker)


if __name__ == '__main__':
    test_call_ops_dump()
    print("\n=== Call Operations Tests Completed Successfully ===")
