# SRAM Module

## Summary

This module implements SRAM (Static Random Access Memory) for Assassyn's IR. SRAM provides immediate, synchronous memory access with single-cycle read/write operations. Unlike DRAM, SRAM operates as on-chip memory with deterministic timing and no request/response cycles. The module extends `MemoryBase` to provide SRAM-specific functionality including read data buffering and mutual exclusion constraints between read and write operations.

## Exposed Interfaces

### `class SRAM`

SRAM memory module that extends MemoryBase for on-chip synchronous memory simulation.

**Purpose:** This class simulates an on-chip SRAM module that provides immediate data access with single-cycle read/write operations. SRAM is suitable for high-performance applications requiring deterministic timing and low latency memory access.

**Inheritance:** Extends `MemoryBase` from [base.py](./base.py)

**Additional Member Fields:**
- `dout: RegArray` - Register buffer that holds the result of read operations

### `def __init__(self, width: int, depth: int, init_file: str | None)`

Initialize SRAM module with read data buffer.

**Parameters:**
- `width: int` - Width of memory in bits (must be positive integer)
- `depth: int` - Depth of memory in words (must be positive integer and power of 2)
- `init_file: str | None` - Path to initialization file for simulation (can be None)

**Returns:** None

**Explanation:**
This constructor calls the parent `MemoryBase.__init__()` method to inherit base memory functionality, then creates an additional `dout` register buffer. The `dout` buffer is implemented as a `RegArray(UInt(width), 1)` to hold the result of read operations, providing a single-word output buffer for the most recently read data.

### `def build(self, we, re, addr, wdata)`

Build the SRAM module with combinational logic for synchronous memory operations.

**Parameters:**
- `we: Value` - Write enable signal
- `re: Value` - Read enable signal
- `addr: Value` - Address signal  
- `wdata: Value` - Write data signal

**Returns:** None

**Explanation:**
This method implements the core SRAM functionality using the `@combinational` decorator from [downstream.py](../module/downstream.md). SRAM provides immediate, synchronous memory access with the following behavior:

1. **Signal Assignment:** All input signals are stored as instance attributes for memory operations
2. **Mutual Exclusion:** Uses `assume(~(we & re))` from [intrinsic.py](../expr/intrinsic.md) to enforce that read and write operations cannot be enabled simultaneously
3. **Write Operation:** When `we` is enabled, writes `wdata` to `_payload[addr]` using conditional execution
4. **Read Operation:** When `re` is enabled, reads `_payload[addr]` and stores the result in `dout[0]` for downstream modules to access

**Technical Details:**
- Uses `Condition` blocks for conditional execution of read/write operations
- Enforces mutual exclusion between read and write operations using `assume` intrinsic
- Provides immediate data access without request/response cycles
- Read data is buffered in `dout` register for downstream module consumption
- Follows the combinational downstream module pattern for same-cycle signal processing

**Design Rationale:**
The mutual exclusion constraint ensures proper memory behavior by preventing simultaneous read and write operations to the same memory location, which could lead to undefined behavior or data corruption. The `dout` buffer allows downstream modules to access read data in the same cycle, enabling efficient pipeline operation.

## Internal Helpers

### `def __repr__(self)`

Provide string representation for debugging.

**Returns:** `str` - String representation with module type identifier

**Explanation:**
Returns a debug-friendly string representation using the internal `_repr_impl()` method with the identifier 'memory.SRAM' for clear module identification in logs and debugging output.