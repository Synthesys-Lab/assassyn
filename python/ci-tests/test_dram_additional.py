"""Additional test cases for DRAM simulator backend."""

import assassyn
from assassyn.frontend import *
from assassyn import backend
from assassyn import utils
from assassyn.ir.module.downstream import Downstream, combinational
from assassyn.ir.expr.intrinsic import (
    send_read_request, send_write_request, 
    read_request_succ, write_request_succ,
    has_mem_resp, get_mem_resp
)


class SimpleDriver(Module):
    """Simple driver for basic DRAM operations."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, dram):
        """Perform simple read/write operations."""
        # Write operation
        addr = Int(9)(0x10)  # Address 16
        data = Int(32)(0x12345678)  # Test data
        send_write_request(dram, addr, data)
        
        # Read operation
        addr = Int(9)(0x20)  # Address 32
        send_read_request(dram, addr)
        
        return write_request_succ(dram), read_request_succ(dram)


class MultiDRAMDriver(Module):
    """Driver that uses multiple DRAM modules."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        """Use two DRAM modules simultaneously."""
        dram1 = DRAM(32, 256, None)
        dram2 = DRAM(32, 256, None)
        
        # Write to both DRAMs
        addr1 = Int(9)(0x10)
        addr2 = Int(9)(0x20)
        data = Int(32)(0xABCDEF00)
        
        send_write_request(dram1, addr1, data)
        send_write_request(dram2, addr2, data)
        
        # Read from both DRAMs
        send_read_request(dram1, addr1)
        send_read_request(dram2, addr2)
        
        return dram1, dram2, write_request_succ(dram1), write_request_succ(dram2)


class ResponseHandler(Downstream):
    """Handler for DRAM responses."""

    def __init__(self):
        super().__init__(ports={})
    
    @downstream.combinational
    def build(self, dram, write_succ, read_succ):
        """Handle responses from a single DRAM."""
        assume(write_succ & read_succ)
        
        with Condition(has_mem_resp(dram)):
            resp = get_mem_resp(dram)
            data = resp[0:32]
            addr = resp[32:32+9]
            log('Response: data={}, addr={}', data, addr)


class MultiResponseHandler(Downstream):
    """Handler for multiple DRAM responses."""

    def __init__(self):
        super().__init__(ports={})
    
    @downstream.combinational
    def build(self, dram1, dram2, write_succ1, write_succ2):
        """Handle responses from multiple DRAMs."""
        assume(write_succ1 & write_succ2)
        
        # Check responses from both DRAMs
        with Condition(has_mem_resp(dram1)):
            resp1 = get_mem_resp(dram1)
            log('DRAM1 Response: {}', resp1[0:32])
            
        with Condition(has_mem_resp(dram2)):
            resp2 = get_mem_resp(dram2)
            log('DRAM2 Response: {}', resp2[0:32])


def test_basic_dram_operations():
    """Test basic DRAM read/write operations."""
    sys = SysBuilder('basic_dram')
    with sys:
        dram = DRAM(32, 256, None)
        driver = SimpleDriver()
        write_succ, read_succ = driver.build(dram)
        
        handler = ResponseHandler()
        handler.build(dram, write_succ, read_succ)

    config = backend.config(sim_threshold=100, idle_threshold=50, verilog=False)
    simulator_path, _ = backend.elaborate(sys, **config)
    
    raw = utils.run_simulator(simulator_path)
    print("Basic DRAM test completed successfully")


def test_multiple_dram_modules():
    """Test multiple DRAM modules in the same system."""
    sys = SysBuilder('multi_dram')
    with sys:
        driver = MultiDRAMDriver()
        dram1, dram2, write_succ1, write_succ2 = driver.build()
        
        handler = MultiResponseHandler()
        handler.build(dram1, dram2, write_succ1, write_succ2)

    config = backend.config(sim_threshold=100, idle_threshold=50, verilog=False)
    simulator_path, _ = backend.elaborate(sys, **config)
    
    raw = utils.run_simulator(simulator_path)
    print("Multiple DRAM test completed successfully")


def test_dram_error_conditions():
    """Test DRAM error conditions and edge cases."""
    sys = SysBuilder('dram_errors')
    with sys:
        dram = DRAM(32, 256, None)
        
        # Test with invalid address (should still work but test edge case)
        invalid_addr = Int(9)(0x1FF)  # Max address
        data = Int(32)(0xDEADBEEF)
        
        send_write_request(dram, invalid_addr, data)
        send_read_request(dram, invalid_addr)
        
        write_succ = write_request_succ(dram)
        read_succ = read_request_succ(dram)
        
        handler = ResponseHandler()
        handler.build(dram, write_succ, read_succ)

    config = backend.config(sim_threshold=50, idle_threshold=25, verilog=False)
    simulator_path, _ = backend.elaborate(sys, **config)
    
    raw = utils.run_simulator(simulator_path)
    print("DRAM error conditions test completed successfully")


if __name__ == "__main__":
    test_basic_dram_operations()
    test_multiple_dram_modules()
    test_dram_error_conditions()
