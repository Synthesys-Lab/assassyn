# Simulator Utility Functions

This module provides utility functions for simulator code generation, including name conversion, data type mapping, and immediate value handling. These functions ensure consistent naming conventions and proper type conversions between Assassyn IR and Rust code.

## Section 0. Summary

The simulator utilities module provides essential helper functions for the simulator code generation pipeline. It handles the conversion of Assassyn data types to Rust types, generates appropriate names for FIFOs and other entities, and converts immediate values to Rust literals. These utilities ensure that the generated Rust code follows proper naming conventions and type safety requirements.

## Section 1. Exposed Interfaces

### camelize

```python
def camelize(name: str) -> str:
    """Convert a name to camelCase.

    This matches the Rust function in src/backend/simulator/utils.rs
    """
```

**Explanation:**

This function converts underscore-separated names to camelCase format, which is commonly used in Rust code. The function iterates through each character in the input string, capitalizing the first letter after each underscore and removing the underscores. This ensures consistent naming conventions between Python and Rust code generation.

### dtype_to_rust_type

```python
def dtype_to_rust_type(dtype: DType) -> str:
    """Convert an Assassyn data type to a Rust type.

    This matches the Rust function in src/backend/simulator/utils.rs
    """
```

**Explanation:**

This function maps Assassyn data types to their corresponding Rust types. It handles several important conversions:

1. **Integer types**: Converts to `u8`, `u16`, `u32`, `u64` for unsigned integers and `i8`, `i16`, `i32`, `i64` for signed integers. The bit width is rounded up to the next power of 2 for standard Rust integer types.

2. **Boolean types**: Single-bit values are converted to `bool`.

3. **Large integers**: Values larger than 64 bits are converted to `BigUint` or `BigInt` for arbitrary precision arithmetic.

4. **Void types**: Converted to `Box<EventKind>` for event handling.

5. **Array types**: Converted to Rust fixed-size arrays with the appropriate element type and size.

The function ensures that all Assassyn data types have proper Rust representations, maintaining type safety and compatibility with the Rust runtime.

### int_imm_dumper_impl

```python
def int_imm_dumper_impl(ty: DType, value: int) -> str:
    """Generate Rust code for integer immediate values.

    This matches the Rust function in src/backend/simulator/elaborate.rs
    """
```

**Explanation:**

This function generates Rust code for integer immediate values, handling different bit widths and signedness appropriately. It implements several important behaviors:

1. **Boolean values**: Single-bit values are converted to `true` or `false` literals.

2. **Small integers**: Values up to 64 bits are converted to Rust integer literals with appropriate type suffixes.

3. **Large integers**: Values larger than 64 bits are converted using the `ValueCastTo` trait to ensure proper type conversion and avoid overflow issues.

The function ensures that immediate values are properly represented in the generated Rust code, maintaining type safety and avoiding potential overflow or underflow issues.

### fifo_name

```python
def fifo_name(fifo: Port):
    """Generate a name for a FIFO.

    This matches the Rust macro in src/backend/simulator/elaborate.rs
    """
```

**Explanation:**

This function generates a consistent name for FIFO instances based on the module and port names. The naming convention follows the pattern `{module_name}_{port_name}`, ensuring that each FIFO has a unique identifier in the simulator context. This naming scheme is used throughout the simulator generation pipeline to reference FIFOs in the generated Rust code.

The function demonstrates the importance of consistent naming conventions in code generation, ensuring that references to FIFOs are properly resolved and that the generated code is maintainable and debuggable.

## Section 2. Internal Helpers

The utility functions in this module are primarily simple helper functions that don't require complex internal implementations. Each function is designed to be self-contained and focused on a specific aspect of the simulator generation process.

The functions work together to provide a comprehensive set of utilities for:
- Name conversion and formatting
- Type mapping between Assassyn and Rust
- Immediate value handling
- FIFO naming conventions

These utilities form the foundation for the simulator code generation pipeline, ensuring that all generated code follows consistent conventions and maintains proper type safety.
