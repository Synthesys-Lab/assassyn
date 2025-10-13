# Array Expression Generation

This module provides Verilog code generation for array and FIFO operations, including array read/write operations and FIFO push/pop operations.

## Summary

The array expression generation module handles the conversion of Assassyn array and FIFO operations into Verilog code. It manages array access patterns, FIFO communication, and integrates with the module's expose mechanism to ensure proper signal routing and port generation.

## Exposed Interfaces

### `codegen_array_read`

```python
def codegen_array_read(dumper, expr: ArrayRead) -> Optional[str]:
    """Generate code for array read operations."""
```

**Explanation**

This function generates Verilog code for array read operations. It handles two different cases:

1. **SRAM Payload Arrays**: When reading from an SRAM module's payload array, it generates a direct assignment to the memory data output signal (`self.mem_dataout`). This is used for SRAM modules where the array is the internal memory payload.

2. **Regular Arrays**: For regular arrays, it generates array access code using the array's read interface:
   - Extracts the array index, handling both constant and variable indices
   - Converts the index to the appropriate bit width if needed
   - Generates array access using the format `self.array_name_q_in[index]`
   - Handles record types differently from scalar types
   - Applies type casting for non-record types

The function also calls `dumper.expose('array', expr)` to register the array operation with the module's expose mechanism, ensuring proper port generation and signal routing.

**Project-specific Knowledge Required**:
- Understanding of [array read operations](/python/assassyn/ir/expr/array.md)
- Knowledge of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Understanding of [expose mechanism](/python/assassyn/codegen/verilog/design.md)
- Reference to [type casting utilities](/python/assassyn/codegen/verilog/utils.md)

### `codegen_array_write`

```python
def codegen_array_write(dumper, expr: ArrayWrite) -> Optional[str]:
    """Generate code for array write operations."""
```

**Explanation**

This function handles array write operations by registering them with the module's expose mechanism. Unlike array reads, array writes don't generate immediate code but instead:

1. **Expose Registration**: Calls `dumper.expose('array', expr)` to register the write operation
2. **Deferred Processing**: The actual write logic is generated later during the cleanup phase by the [cleanup module](/python/assassyn/codegen/verilog/cleanup.md)

This deferred approach allows the cleanup phase to:
- Group multiple writes to the same array
- Generate appropriate port-based write signals
- Handle write data multiplexing
- Create proper write enable and address signals

**Project-specific Knowledge Required**:
- Understanding of [array write operations](/python/assassyn/ir/expr/array.md)
- Knowledge of [expose mechanism](/python/assassyn/codegen/verilog/design.md)
- Understanding of [cleanup phase processing](/python/assassyn/codegen/verilog/cleanup.md)

### `codegen_fifo_push`

```python
def codegen_fifo_push(dumper, expr: FIFOPush) -> Optional[str]:
    """Generate code for FIFO push operations."""
```

**Explanation**

This function handles FIFO push operations by registering them with the module's expose mechanism. Similar to array writes, FIFO pushes don't generate immediate code but instead:

1. **Expose Registration**: Calls `dumper.expose('fifo', expr)` to register the push operation
2. **Deferred Processing**: The actual push logic is generated later during the cleanup phase

The cleanup phase handles FIFO push operations by:
- Combining multiple push predicates
- Generating push valid and data signals
- Creating proper FIFO interface connections
- Handling push data multiplexing when multiple pushes occur

**Project-specific Knowledge Required**:
- Understanding of [FIFO push operations](/python/assassyn/ir/expr/array.md)
- Knowledge of [expose mechanism](/python/assassyn/codegen/verilog/design.md)
- Understanding of [cleanup phase processing](/python/assassyn/codegen/verilog/cleanup.md)

### `codegen_fifo_pop`

```python
def codegen_fifo_pop(dumper, expr: FIFOPop) -> Optional[str]:
    """Generate code for FIFO pop operations."""
```

**Explanation**

This function generates Verilog code for FIFO pop operations. It performs the following steps:

1. **Name Generation**: Generates a unique name for the pop operation using `namify(expr.as_operand())`
2. **FIFO Reference**: Gets the FIFO name using `dump_rval()` to generate the proper signal reference
3. **Expose Registration**: Calls `dumper.expose('fifo_pop', expr)` to register the pop operation
4. **Code Generation**: Returns an assignment that reads from the FIFO's output signal

The generated code assigns the FIFO's output data to a local variable, allowing the popped data to be used in subsequent operations.

**Project-specific Knowledge Required**:
- Understanding of [FIFO pop operations](/python/assassyn/ir/expr/array.md)
- Knowledge of [expose mechanism](/python/assassyn/codegen/verilog/design.md)
- Understanding of [right-hand value generation](/python/assassyn/codegen/verilog/rval.md)
- Reference to [name generation utilities](/python/assassyn/utils.md)

## Internal Helpers

The module uses several utility functions:

- `dump_rval()` from [rval module](/python/assassyn/codegen/verilog/rval.md) for generating signal references
- `dump_type()` and `dump_type_cast()` from [utils module](/python/assassyn/codegen/verilog/utils.md) for type handling
- `unwrap_operand()` and `namify()` from [utils module](/python/assassyn/utils.md) for operand processing and name generation

The array expression generation is integrated into the main expression dispatch system through the [__init__.py](/python/assassyn/codegen/verilog/_expr/__init__.md) module, which routes different expression types to their appropriate code generation functions.

**Project-specific Knowledge Required**:
- Understanding of [expression dispatch system](/python/assassyn/codegen/verilog/_expr/__init__.md)
- Knowledge of [CIRCTDumper integration](/python/assassyn/codegen/verilog/design.md)
- Reference to [array and FIFO expression types](/python/assassyn/ir/expr/array.md)
- Understanding of [credit-based pipeline architecture](/docs/design/arch/arch.md)
