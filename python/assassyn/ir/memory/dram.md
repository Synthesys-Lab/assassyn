# DRAM Module

## Summary

This module implements DRAM (Dynamic Random Access Memory) simulation for Assassyn's IR. Unlike SRAM, DRAM operates as an off-chip memory module that interacts with the on-chip pipeline through asynchronous request/response mechanisms. The module extends `MemoryBase` to provide DRAM-specific functionality using intrinsic functions for memory request handling, as described in the [intrinsics documentation](../expr/intrinsic.md).

## Exposed Interfaces

### `class DRAM`

DRAM memory module that extends MemoryBase for off-chip memory simulation.

**Purpose:** This class simulates an off-chip DRAM module that communicates with the on-chip pipeline through asynchronous memory requests. Unlike SRAM which provides immediate data access, DRAM requires request/response cycles and may have variable latency.

**Inheritance:** Extends `MemoryBase` from [base.py](./base.py)

### `def __init__(self, width: int, depth: int, init_file: str | None)`

Initialize DRAM module with the same interface as MemoryBase.

**Parameters:**
- `width: int` - Width of memory in bits (must be positive integer)
- `depth: int` - Depth of memory in words (must be positive integer and power of 2)
- `init_file: str | None` - Path to initialization file for simulation (can be None)

**Returns:** None

**Explanation:**
This constructor delegates to the parent `MemoryBase.__init__()` method, inheriting all the base memory functionality including parameter validation, address width calculation, and payload array creation. No DRAM-specific initialization is required at construction time.

### `def build(self, we, re, addr, wdata)`

Build the DRAM module with combinational logic for memory request handling.

**Parameters:**
- `we: Value` - Write enable signal
- `re: Value` - Read enable signal  
- `addr: Value` - Address signal
- `wdata: Value` - Write data signal

**Returns:** `(read_succ, write_succ)` - Tuple of boolean values indicating request success

**Explanation:**
This method implements the core DRAM functionality using the `@combinational` decorator from [downstream.py](../module/downstream.md). Unlike SRAM which provides immediate data access, DRAM uses asynchronous request/response patterns:

1. **Signal Assignment:** All input signals are stored as instance attributes for use by the memory system
2. **Read Request:** When `re` is enabled, calls `send_read_request(self, re, addr)` from [intrinsic.py](../expr/intrinsic.md) to initiate an asynchronous read operation
3. **Write Request:** When `we` is enabled, calls `send_write_request(self, we, addr, wdata)` to initiate an asynchronous write operation
4. **Success Indication:** Returns boolean values indicating whether each request was successfully sent

The success signals allow downstream modules to determine if requests need to be retried. This follows the asynchronous memory access pattern described in [arch.md](../../../docs/design/arch/arch.md) where memory operations may have variable latency and require proper flow control.

**Technical Details:**
- Uses intrinsic functions `send_read_request` and `send_write_request` for memory system integration
- Requests are only sent when the corresponding enable signal is asserted
- Success/failure indication enables proper error handling and retry logic
- Follows the combinational downstream module pattern for same-cycle signal processing

## Internal Helpers

### `def __repr__(self)`

Provide string representation for debugging.

**Returns:** `str` - String representation with module type identifier

**Explanation:**
Returns a debug-friendly string representation using the internal `_repr_impl()` method with the identifier 'memory.DRAM' for clear module identification in logs and debugging output.