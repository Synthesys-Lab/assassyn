# Array Code Generation Module

This module provides helper functions to generate Rust code for array read and write operations for the simulator backend.

-----

## Exposed Interfaces

```python
def codegen_array_read(node, module_ctx, sys) -> str
def codegen_array_write(node, module_ctx, sys, module_name) -> str
```

-----

## Array Read Generation

The `codegen_array_read` function generates a Rust expression to read from an array by accessing its `payload` field at a given index.

  * **Generated Code**: `sim.<array_name>.payload[<index> as usize].clone()`

-----

## Array Write Generation

The `codegen_array_write` function generates a Rust code block to write to an array. It pushes a timestamped `ArrayWrite` object-containing the value, index, and writer's info-onto the array's `write_port`.

  * **Generated Code**:
    ```rust
    {
        let stamp = sim.stamp - sim.stamp % 100 + 50;
        sim.<array_name>.write_port.push(
          ArrayWrite::new(stamp, <index> as usize, <value>.clone(), "<module_name>", <port_id>));
    }
    ```
