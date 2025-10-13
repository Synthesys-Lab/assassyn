# Memory Base

## Summary

This module implements the base interface for memory modules in Assassyn's IR. It provides a common foundation for both SRAM and DRAM implementations, handling the basic memory interface including address decoding, data storage, and signal management. The module extends the [Downstream](../module/downstream.md) class to support combinational memory operations within the credit-based pipeline architecture described in [arch.md](../../../docs/design/arch/arch.md).

## Exposed Interfaces

### `class MemoryBase`

Base class for memory modules that provides common functionality for SRAM and DRAM implementations.

**Purpose:** This class serves as the foundation for all memory modules in Assassyn, providing a unified interface for memory operations while allowing specialized implementations for different memory types.

**Member Fields:**
- `width: int` - Width of the memory in bits
- `depth: int` - Depth of the memory in words (must be power of 2)
- `init_file: str | None` - Path to initialization file for simulation (can be None)
- `we: Value` - Write enable signal (combinational input)
- `re: Value` - Read enable signal (combinational input)  
- `addr: Value` - Address signal (combinational input)
- `wdata: Value` - Write data signal (combinational input)
- `addr_width: int` - Width of the address in bits (derived as log2(depth))
- `_payload: Array` - Array holding the memory contents (private, not for direct access)

### `def __init__(self, width: int, depth: int, init_file: str | None)`

Initialize memory base class with validation and setup.

**Parameters:**
- `width: int` - Width of memory in bits (must be positive integer)
- `depth: int` - Depth of memory in words (must be positive integer and power of 2)
- `init_file: str | None` - Path to initialization file for simulation (can be None)

**Returns:** None

**Explanation:**
This constructor validates all input parameters and sets up the memory module infrastructure. It enforces that depth must be a power of 2 to enable efficient address decoding using log2 operations. The constructor creates a `RegArray` instance with the specified width and depth to serve as the memory payload, using the instance name for proper identification in generated code. All signal attributes are initialized to None and will be assigned during the `build()` method of concrete implementations.

The payload array is created using `RegArray(Bits(width), depth)` from [ir/array.py](../array.py) to emulate register-based memory behavior. The `_payload` field is marked as private (prefixed with underscore) as it should not be accessed directly by users - memory operations should go through the proper interface methods.

## Internal Helpers

No internal helper functions are implemented in this base class. All functionality is provided through the constructor and will be extended by concrete memory implementations (SRAM and DRAM) through their respective `build()` methods.