# Verilog Utility Functions

This module provides utility functions for the Verilog backend, including type conversion, SRAM handling, and common operations needed during Verilog code generation.

## Summary

The utils module contains essential helper functions used throughout the Verilog code generation process. It provides type conversion utilities, SRAM information extraction, expression processing helpers, and common constants like the Verilog header template.

## Exposed Interfaces

### `dump_type`

```python
def dump_type(ty: DType) -> str:
    """Dump a type to a string."""
```

**Explanation**

This function converts Assassyn data types into their corresponding Verilog type representations. It handles the following type mappings:

1. **Int types**: Converts to `SInt(bits)` for signed integers
2. **UInt types**: Converts to `UInt(bits)` for unsigned integers  
3. **Bits types**: Converts to `Bits(bits)` for bit vectors
4. **Record types**: Converts to `Bits(bits)` using the total bit width
5. **Slice types**: Calculates width from start/stop indices and returns `Bits(width)`

The function is used extensively throughout the codebase for generating proper Verilog type declarations and type conversions.

**Project-specific Knowledge Required**:
- Understanding of [Assassyn data types](/python/assassyn/ir/dtype.md)
- Knowledge of [Verilog type system](/docs/design/lang/type.md)

### `dump_type_cast`

```python
def dump_type_cast(ty: DType, bits: int = None) -> str:
    """Dump a type to a string."""
```

**Explanation**

This function generates type casting expressions for converting between different Verilog types. It returns the appropriate casting method name based on the target type:

1. **Int types**: Returns `as_sint(bits)` for signed integer casting
2. **UInt types**: Returns `as_uint(bits)` for unsigned integer casting
3. **Bits/Record types**: Returns `as_bits(bits)` for bit vector casting

The `bits` parameter allows specifying a custom bit width for the cast operation. If not provided, it uses the type's natural bit width.

**Project-specific Knowledge Required**:
- Understanding of [type casting in Verilog](/python/assassyn/codegen/verilog/rval.md)
- Knowledge of [arithmetic expression generation](/python/assassyn/codegen/verilog/_expr/arith.md)

### `get_sram_info`

```python
def get_sram_info(node: SRAM) -> dict:
    """Extract SRAM-specific information."""
```

**Explanation**

This function extracts essential information from an SRAM module for Verilog generation. It returns a dictionary containing:

1. **array**: The underlying array object (`node._payload`)
2. **init_file**: Initialization file path for the SRAM
3. **width**: Data width of the SRAM
4. **depth**: Depth (number of entries) of the SRAM

This information is used by other modules to generate appropriate SRAM interface signals and memory control logic.

**Project-specific Knowledge Required**:
- Understanding of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Knowledge of [memory interface generation](/python/assassyn/codegen/verilog/cleanup.md)

### `extract_sram_params`

```python
def extract_sram_params(node: SRAM) -> dict:
    """Extract common SRAM parameters from an SRAM module.

    Args:
        sram: SRAM module object

    Returns:
        dict: Dictionary containing array_name, data_width, and addr_width
    """
```

**Explanation**

This function provides a higher-level interface for extracting SRAM parameters needed for Verilog generation. It combines `get_sram_info()` with additional processing to provide:

1. **sram_info**: The raw SRAM information dictionary
2. **array**: The underlying array object
3. **array_name**: Generated name for the array
4. **data_width**: Width of data elements in bits
5. **addr_width**: Width of address bus (minimum 1 bit)

The function ensures that address width is at least 1 bit even for single-element arrays.

**Project-specific Knowledge Required**:
- Understanding of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Knowledge of [name generation utilities](/python/assassyn/utils.md)

### `find_wait_until`

```python
def find_wait_until(module: Module) -> Optional[Intrinsic]:
    """Find the WAIT_UNTIL intrinsic in a module if it exists."""
```

**Explanation**

This function searches through a module's body to find the `WAIT_UNTIL` intrinsic, which is used in the credit-based pipeline architecture to control module execution timing. It returns the intrinsic if found, or `None` if not present.

The function is used to determine whether a module has explicit wait conditions that need to be incorporated into the execution logic.

**Project-specific Knowledge Required**:
- Understanding of [intrinsic operations](/python/assassyn/ir/expr/intrinsic.md)
- Knowledge of [credit-based pipeline architecture](/docs/design/arch/arch.md)

### `ensure_bits`

```python
def ensure_bits(expr_str: str) -> str:
    """Ensure an expression is of Bits type, converting if necessary."""
```

**Explanation**

This function ensures that a Verilog expression string represents a Bits type, performing necessary conversions. It handles several cases:

1. **UInt to Bits conversion**: Converts `UInt(width)(value)` to `Bits(width)(value)`
2. **Already Bits**: Returns unchanged if already a Bits type
3. **Already converted**: Returns unchanged if `.as_bits()` is already present
4. **Control signals**: Returns unchanged for common control signal patterns
5. **Default conversion**: Adds `.as_bits()` to other expressions

This function is used to ensure type consistency in Verilog signal assignments and expressions.

**Project-specific Knowledge Required**:
- Understanding of [Verilog type system](/docs/design/lang/type.md)
- Knowledge of [signal generation](/python/assassyn/codegen/verilog/cleanup.md)

## Internal Constants

### `HEADER`

The `HEADER` constant contains the standard Python CIRCT imports plus a reference to shared runtime helpers used in generated Verilog code:

- **Imports**: Essential CIRCT modules and types
- **Runtime wrappers**: Imports `FIFO`, `TriggerCounter`, and `build_register_file` from `assassyn.pycde_wrapper`, ensuring generated designs reuse the shared FIFO, trigger counter, and register-file primitives instead of emitting bespoke definitions

This header is included in all generated Verilog modules to provide the necessary infrastructure for the credit-based pipeline architecture while avoiding duplicate class definitions.

**Project-specific Knowledge Required**:
- Understanding of [CIRCT framework](/docs/design/internal/pipeline.md)
- Knowledge of [FIFO implementation](/docs/design/internal/pipeline.md)
- Reference to [trigger counter design](/docs/design/internal/pipeline.md)

The utility functions in this module are fundamental to the Verilog code generation process, providing the necessary type conversions, memory handling, and infrastructure components required for generating synthesizable Verilog code.
