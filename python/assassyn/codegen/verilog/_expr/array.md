# Array Expression Generation

This module provides Verilog code generation for array and FIFO operations, including array read/write operations and FIFO push/pop operations. All public helpers in this file now declare `dumper: CIRCTDumper` in their signatures and are wrapped with the project-wide [`@enforce_type`](../../../utils/enforce_type.md) decorator, ensuring callers provide the expected dumper implementation and expression types at runtime.

## Summary

The array expression generation module handles the conversion of Assassyn array and FIFO operations into Verilog code. It manages array access patterns, FIFO communication, and feeds module metadata so the cleanup pass can generate the required ports and handshakes.

## Exposed Interfaces

### `codegen_array_read`

```python
@enforce_type
def codegen_array_read(dumper: CIRCTDumper, expr: ArrayRead) -> Optional[str]:
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

Metadata analysis captures array reads ahead of time, so the helper simply returns the body text.

**Runtime contract**: The helper expects a fully initialised `CIRCTDumper` and validates both arguments via `@enforce_type`. Passing any other dumper stub raises a `TypeError` before code generation begins.

**Project-specific Knowledge Required**:
- Understanding of [array read operations](/python/assassyn/ir/expr/array.md)
- Knowledge of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Understanding of [expose mechanism](/python/assassyn/codegen/verilog/design.md)
- Reference to [type casting utilities](/python/assassyn/codegen/verilog/utils.md)

### `codegen_array_write`

```python
@enforce_type
def codegen_array_write(dumper: CIRCTDumper, expr: ArrayWrite) -> Optional[str]:
    """Generate code for array write operations."""
```

**Explanation**

Array writes are metadata-only. The IR analysis pre-pass records every `ArrayWrite` encountered in module metadata so the cleanup phase can build the final port handshakes. As a result the emitter simply validates its arguments and returns `None`.

**Runtime contract**: The helper validates its arguments via `@enforce_type`, protecting against accidental invocation with a non-`CIRCTDumper` dumper or mismatched expression node. See [Type Enforcement](../../../utils/enforce_type.md) for decorator details.

**Project-specific Knowledge Required**:
- Understanding of [array write operations](/python/assassyn/ir/expr/array.md)
- Knowledge of [expose mechanism](/python/assassyn/codegen/verilog/design.md)
- Understanding of [cleanup phase processing](/python/assassyn/codegen/verilog/cleanup.md)

### `codegen_fifo_push`

```python
@enforce_type
def codegen_fifo_push(dumper: CIRCTDumper, expr: FIFOPush) -> Optional[str]:
    """Generate code for FIFO push operations."""
```

**Explanation**

This function handles FIFO push operations without emitting Verilog immediately. Instead, it:

1. **Metadata Collection**: Records a FIFO push entry in the module's metadata (see [metadata module](/python/assassyn/codegen/verilog/metadata.md)), capturing the producing module, the `FIFOPush` expression, and the predicate carry supplied by the base `Expr` (`expr.meta_cond`). This avoids redundant expression walking while preserving the control context for later wiring.
2. **Deferred Processing**: Defers signal emission to the cleanup phase.

The cleanup phase handles FIFO push operations by:
- Combining multiple push predicates
- Generating push valid and data signals
- Creating proper FIFO interface connections
- Handling push data multiplexing when multiple pushes occur

**Runtime contract**: `@enforce_type` enforces that callers pass a `CIRCTDumper` instance and a `FIFOPush` expression, ensuring metadata tracking happens on the correct dumper implementation.

**Project-specific Knowledge Required**:
- Understanding of [FIFO push operations](/python/assassyn/ir/expr/array.md)
- Familiarity with [metadata collection](/python/assassyn/codegen/verilog/metadata.md)
- Understanding of [cleanup phase processing](/python/assassyn/codegen/verilog/cleanup.md)

### `codegen_fifo_pop`

```python
@enforce_type
def codegen_fifo_pop(dumper: CIRCTDumper, expr: FIFOPop) -> Optional[str]:
    """Generate code for FIFO pop operations."""
```

**Explanation**

This function generates Verilog code for FIFO pop operations. It performs the following steps:

1. **Name Generation**: Generates a unique name for the pop operation using `namify(expr.as_operand())`
2. **FIFO Reference**: Gets the FIFO name using `dump_rval()` to generate the proper signal reference
3. **Metadata Collection**: Records a FIFO pop entry in the module's metadata with the current module and predicate carry from the expression (`expr.meta_cond`), enabling downstream consumers to reason about handshake readiness under the same conditions as the original IR.
4. **Code Generation**: Returns an assignment that reads from the FIFO's output signal

The generated code assigns the FIFO's output data to a local variable, allowing the popped data to be used in subsequent operations.

**Runtime contract**: The helper enforces the `CIRCTDumper` contract through `@enforce_type`, preventing downstream code from using incomplete dumper stubs that would break metadata bookkeeping.

**Project-specific Knowledge Required**:
- Understanding of [FIFO pop operations](/python/assassyn/ir/expr/array.md)
- Familiarity with [metadata collection](/python/assassyn/codegen/verilog/metadata.md)
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
