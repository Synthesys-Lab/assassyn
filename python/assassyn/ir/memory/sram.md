# SRAM Module

## Design Documents

- [Memory System Architecture](../../../docs/design/arch/memory.md) - Memory system design including SRAM
- [Intrinsic Operations Design](../../../docs/design/lang/intrinsics.md) - Intrinsic operations architecture
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Memory Base](./base.md) - Base memory implementation
- [Intrinsic Operations](../expr/intrinsic.md) - Intrinsic function operations
- [Downstream Module](../module/downstream.md) - Downstream module implementation
- [Array Operations](../expr/array.md) - Array read/write operations

## Summary

This module implements SRAM (Static Random Access Memory) for Assassyn's IR. SRAM provides immediate, synchronous memory access with single-cycle read/write operations. Unlike DRAM, SRAM operates as on-chip memory with deterministic timing and no request/response cycles. The module extends `MemoryBase` to provide SRAM-specific functionality including read data buffering and mutual exclusion constraints between read and write operations.

## Exposed Interfaces

### `class SRAM`

SRAM memory module that extends MemoryBase for on-chip synchronous memory simulation.

**Purpose:** This class simulates an on-chip SRAM module that provides immediate data access with single-cycle read/write operations. SRAM is suitable for high-performance applications requiring deterministic timing and low latency memory access.

**Inheritance:** Extends `MemoryBase` from [base.py](./base.py)

**Additional Member Fields:**
- `dout: RegArray` - Register buffer that holds the result of read operations (uses Bits type for compatibility with array read operations)

### `def __init__(self, width: int, depth: int, init_file: str | None)`

Initialize SRAM module with read data buffer.

**Parameters:**
- `width: int` - Width of memory in bits (must be positive integer)
- `depth: int` - Depth of memory in words (must be positive integer and power of 2)
- `init_file: str | None` - Path to initialization file for simulation (can be None)

**Returns:** None

**Explanation:**
This constructor calls the parent `MemoryBase.__init__()` method to inherit base memory functionality, then creates an additional `dout` register buffer. The `dout` buffer is implemented as a `RegArray(Bits(width), 1)` to hold the result of read operations, providing a single-word output buffer for the most recently read data. Using `Bits` type ensures compatibility with array read operations that return raw bit values.

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

**SRAM Read Data Timing:** The relationship between read enable timing and `dout` buffer update:
- **Immediate Update**: When `re` is enabled, the `dout` buffer is updated immediately in the same cycle
- **Last Cycle Enable**: The `dout` buffer contains the data from the last cycle when `re` was enabled
- **Combinational Read**: Read operations are combinational, providing immediate data access
- **Buffer Persistence**: The `dout` buffer retains its value until the next read operation

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

## Performance Characteristics

**SRAM Module Performance Notes:**
- **Latency**: 1 cycle for both read and write operations
- **Throughput**: Single-cycle access enables high throughput for sequential access patterns
- **Resource Usage**: Consumes significant hardware resources proportional to width × depth
- **Power Consumption**: Higher power consumption compared to DRAM due to continuous power requirements
- **Timing Constraints**: Provides deterministic timing with no variable latency
- **Access Patterns**: Optimized for random access patterns with immediate data availability