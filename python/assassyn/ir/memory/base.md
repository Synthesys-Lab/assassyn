# Memory Base

## Design Documents

- [Memory System Architecture](../../../docs/design/arch/memory.md) - Memory system design including SRAM and DRAM
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system

## Related Modules

- [Downstream Module](../module/downstream.md) - Downstream module implementation
- [Array Operations](../expr/array.md) - Array read/write operations
- [DRAM Module](./dram.md) - DRAM implementation
- [SRAM Module](./sram.md) - SRAM implementation

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
- `_payload: Array` - Array holding the memory contents (private, not for direct access, owned by the memory instance)

### `def __init__(self, width: int, depth: int, init_file: str | None)`

Initialize memory base class with validation and setup.

**Parameters:**
- `width: int` - Width of memory in bits (must be positive integer)
- `depth: int` - Depth of memory in words (must be positive integer and power of 2)
- `init_file: str | None` - Path to initialization file for simulation (can be None)

**Returns:** None

**Explanation:**
This constructor validates all input parameters and sets up the memory module infrastructure. It enforces that depth must be a power of 2 to enable efficient address decoding using log2 operations. The constructor creates a `RegArray` instance with the specified width and depth to serve as the memory payload, using the instance name for proper identification in generated code. All signal attributes are initialized to None and will be assigned during the `build()` method of concrete implementations.

**Address Width Derivation Logic:** The address width is calculated as `addr_width = log2(depth)` because:
1. **Power-of-2 Constraint**: Depth must be a power of 2 to enable efficient address decoding
2. **Hardware Implementation**: Power-of-2 depths allow for simple address decoding using bit selection
3. **Efficient Addressing**: The log2 operation provides the exact number of address bits needed
4. **Hardware Optimization**: This constraint enables efficient hardware implementation with minimal address decoding logic

**Memory Initialization File Format:** The `init_file` parameter supports initialization files for simulation:
- **File Format**: Hexadecimal format (`.hex` files) with one value per line
- **Byte Ordering**: Little-endian byte ordering for multi-byte values
- **Address Mapping**: Values are loaded sequentially starting from address 0
- **Simulation Only**: Initialization files are used only during simulation, not in hardware generation

The payload array is created using `RegArray(Bits(width), depth, owner=self)` from [ir/array.py](../array.py) so the owning memory instance is recorded directly. Using `Bits` ensures compatibility with array read operations that return raw bit values. The `_payload` field is marked as private (prefixed with underscore) as it should not be accessed directly by users—memory operations must go through the proper interface methods. Downstream passes rely on `Array.is_payload(memory)` instead of direct identity checks to route payload arrays through dedicated SRAM/DRAM plumbing.

## Internal Helpers

No internal helper functions are implemented in this base class. All functionality is provided through the constructor and will be extended by concrete memory implementations (SRAM and DRAM) through their respective `build()` methods.

## Performance Characteristics

**Memory Module Performance Notes:**
- **Latency**: Memory access latency depends on the specific implementation (SRAM: 1 cycle, DRAM: variable)
- **Throughput**: Memory throughput is limited by the width parameter and access patterns
- **Resource Usage**: Memory modules consume significant hardware resources proportional to width × depth
- **Power Consumption**: Memory modules are major power consumers in hardware designs
- **Timing Constraints**: Memory modules may impose timing constraints on the overall system design
