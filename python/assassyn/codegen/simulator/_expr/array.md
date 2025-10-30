# Array Code Generation Module

This module provides helper functions to generate Rust code for array read and write operations for the simulator backend. These functions are part of the [simulator code generation pipeline](../simulator.md) and handle the translation of [ArrayRead](../../../ir/expr/array.md) and [ArrayWrite](../../../ir/expr/array.md) IR nodes into executable Rust simulation code.

The generated code follows the [simulator architecture](../simulator.md) with half-cycle timing for register updates and port-based write management for multi-writer arrays.

## Exposed Interfaces

### codegen_array_read

```python
def codegen_array_read(node, module_ctx) -> str:
    """Generate Rust code for array read operations.
    
    Generates a Rust expression that reads from an array's payload at a given index.
    The generated code directly accesses the array's data without any timing considerations
    since reads are combinational in the simulator.
    
    @param node: ArrayRead IR node containing the array and index to read from
    @param module_ctx: Current module context for variable resolution
    @return: Rust expression string for reading from the array
    """
```

**Generated Code**: `sim.<array_name>.payload[<index> as usize].clone()`

**Explanation**: This function generates a simple array access expression that reads from the `payload` field of the array structure. The `clone()` ensures the value is copied for use in the simulation. The index is cast to `usize` as required by Rust's Vec indexing.

### codegen_array_write

```python
def codegen_array_write(node, module_ctx, module_name) -> str:
    """Generate Rust code for array write operations with port indexing.
    
    Generates a Rust code block that writes to an array using the port-based write system.
    The write is timestamped for the half-cycle timing mechanism and uses a port index
    assigned by the port manager for optimal performance.
    
    @param node: ArrayWrite IR node containing array, index, and value to write
    @param module_ctx: Current module context for variable resolution  
    @param module_name: Name of the module performing the write (for port assignment)
    @return: Rust code block string for writing to the array
    """
```

**Generated Code**:
```rust
{
    let stamp = sim.stamp - sim.stamp % 100 + 50;
    let write = ArrayWrite::new(stamp, <index> as usize,
                               <value>.clone(), "<module_name>");
    sim.<array_name>.write(<port_idx>, write);
}
```

**Explanation**: This function generates a code block that creates a timestamped write operation. The timestamp calculation (`sim.stamp - sim.stamp % 100 + 50`) aligns the write to the half-cycle boundary as described in the [simulator timing model](../simulator.md). The write uses a port index assigned by the [port manager](../port_mapper.md) to enable multiple modules to write to the same array efficiently. The actual write is deferred until the next half-cycle when `tick_registers()` is called.
