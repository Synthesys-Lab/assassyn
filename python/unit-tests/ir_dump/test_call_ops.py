"""Tests for call operation IR nodes: Bind, AsyncCall, FIFOPush, FIFOPop."""

import sys
import os
sys.path.append(os.path.join(os.path.dirname(__file__), '..', '..'))

from assassyn.frontend import (
    # Core types
    Module, SysBuilder, UInt, Int, Bits, Record,
    # Arrays and blocks
    RegArray, Condition, Cycle,
    # Intrinsics
    wait_until, finish, assume, barrier,
    # Logging
    log,
    # Module decorator
    module,
    # Ports and wires
    Port, WireIn, WireOut,
    # External modules
    ExternalSV, external,
)


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
    sys_builder = SysBuilder('call_ops_test')
    
    def test_func():
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
    
    with sys_builder:
        test_func()
    
    sys_repr = repr(sys_builder)
    print(f"\n=== Call Ops Test IR Dump ===")
    print(sys_repr)
    
    # Verify call operations appear
    assert "bind_result =" in sys_repr and "bind" in sys_repr
    assert "async_call" in sys_repr


if __name__ == '__main__':
    test_call_ops_dump()
    print("\n=== Call Operations Tests Completed Successfully ===")
