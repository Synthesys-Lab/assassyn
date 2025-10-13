# Arithmetic Expression Generation

This module provides Verilog code generation for arithmetic and logical operations, including binary operations, unary operations, bit manipulation operations, and type casting operations.

## Summary

The arithmetic expression generation module handles the conversion of Assassyn arithmetic expressions into Verilog code. It supports a wide range of operations including basic arithmetic (add, subtract, multiply, divide), logical operations (and, or, xor), bit manipulation (shift, slice, concatenation), and type casting operations (zero-extension, sign-extension, bitcast).

## Exposed Interfaces

### `codegen_binary_op`

```python
def codegen_binary_op(dumper, expr: BinaryOp) -> Optional[str]:
    """Generate code for binary operations."""
```

**Explanation**

This function generates Verilog code for binary operations. It handles different categories of operations with specific logic:

1. **Shift Operations (SHL, SHR)**: 
   - Uses CIRCT combinational operations (`comb.ShlOp`, `comb.ShrSOp`, `comb.ShrUOp`)
   - Handles bit width mismatches by padding the shift amount
   - Distinguishes between signed and unsigned right shifts

2. **Modulo Operations (MOD)**:
   - Uses CIRCT modulo operations (`comb.ModSOp`, `comb.ModUOp`)
   - Distinguishes between signed and unsigned modulo

3. **Comparative Operations**:
   - Converts operands to unsigned integers for comparison
   - Uses standard comparison operators (==, !=, <, >, <=, >=)

4. **Standard Binary Operations**:
   - Handles type mismatches by casting the right operand to match the left operand
   - Special handling for bitwise AND operations
   - Uses standard arithmetic operators (+, -, *, /, &, |, ^)

The function generates assignments in the format `rval = operation_result` where `rval` is the unique name for the expression result.

**Project-specific Knowledge Required**:
- Understanding of [binary operations](/python/assassyn/ir/expr/arith.md)
- Knowledge of [CIRCT combinational operations](/docs/design/internal/pipeline.md)
- Understanding of [type casting utilities](/python/assassyn/codegen/verilog/utils.md)

### `codegen_unary_op`

```python
def codegen_unary_op(dumper, expr: UnaryOp) -> Optional[str]:
    """Generate code for unary operations."""
```

**Explanation**

This function generates Verilog code for unary operations. It handles two types of unary operations:

1. **Bit Flip (FLIP)**: Generates bitwise NOT operation using `~` operator
2. **Negation**: Generates arithmetic negation using `-` operator

The function ensures proper type casting by applying the target type cast to the result. For bit flip operations, it converts the operand to bits first to ensure proper bitwise operation.

**Project-specific Knowledge Required**:
- Understanding of [unary operations](/python/assassyn/ir/expr/arith.md)
- Knowledge of [type casting utilities](/python/assassyn/codegen/verilog/utils.md)

### `codegen_slice`

```python
def codegen_slice(dumper, expr: Slice) -> Optional[str]:
    """Generate code for slice operations."""
```

**Explanation**

This function generates Verilog code for bit slicing operations. It extracts a range of bits from a larger bit vector using Verilog's slice notation `[start:end+1]`.

The function:
1. Gets the source expression using `dump_rval()`
2. Extracts the start and end indices from the slice expression
3. Generates a slice assignment using Verilog slice syntax

**Project-specific Knowledge Required**:
- Understanding of [slice operations](/python/assassyn/ir/array.md)
- Knowledge of [right-hand value generation](/python/assassyn/codegen/verilog/rval.md)

### `codegen_concat`

```python
def codegen_concat(dumper, expr: Concat) -> Optional[str]:
    """Generate code for concatenation operations."""
```

**Explanation**

This function generates Verilog code for bit concatenation operations. It combines two bit vectors into a single larger bit vector using CIRCT's `BitsSignal.concat()` function.

The function concatenates the most significant bits (msb) and least significant bits (lsb) in the correct order to form the result.

**Project-specific Knowledge Required**:
- Understanding of [concatenation operations](/python/assassyn/ir/expr/arith.md)
- Knowledge of [CIRCT BitsSignal operations](/docs/design/internal/pipeline.md)

### `codegen_cast`

```python
def codegen_cast(dumper, expr: Cast) -> Optional[str]:
    """Generate code for cast operations."""
```

**Explanation**

This function generates Verilog code for type casting operations. It handles three types of casts:

1. **Bitcast (BITCAST)**: Direct type conversion without changing bit values
2. **Zero Extension (ZEXT)**: Extends the value with leading zeros
3. **Sign Extension (SEXT)**: Extends the value with sign bits

The function calculates the required padding and generates appropriate concatenation operations to achieve the desired bit width and sign behavior.

**Project-specific Knowledge Required**:
- Understanding of [cast operations](/python/assassyn/ir/expr/arith.md)
- Knowledge of [type casting utilities](/python/assassyn/codegen/verilog/utils.md)
- Understanding of [CIRCT BitsSignal operations](/docs/design/internal/pipeline.md)

### `codegen_select`

```python
def codegen_select(dumper, expr: Select) -> Optional[str]:
    """Generate code for select operations."""
```

**Explanation**

This function generates Verilog code for conditional selection operations (ternary operator). It uses CIRCT's `Mux` construct to select between two values based on a condition.

The function handles type mismatches between the true and false values by casting the false value to match the true value's type.

**Project-specific Knowledge Required**:
- Understanding of [select operations](/python/assassyn/ir/expr/arith.md)
- Knowledge of [CIRCT Mux operations](/docs/design/internal/pipeline.md)

### `codegen_select1hot`

```python
def codegen_select1hot(dumper, expr: Select1Hot) -> Optional[str]:
    """Generate code for 1-hot select operations."""
```

**Explanation**

This function generates Verilog code for one-hot selection operations, which select one value from multiple options based on a one-hot encoded condition.

The function handles two cases:
1. **Single value**: Returns the value directly
2. **Multiple values**: Generates a multiplexer chain using CIRCT's `Mux` construct

For multiple values, it creates a selector by finding the index of the set bit in the one-hot condition and uses that to select the appropriate value.

**Project-specific Knowledge Required**:
- Understanding of [one-hot selection operations](/python/assassyn/ir/expr/arith.md)
- Knowledge of [CIRCT Mux operations](/docs/design/internal/pipeline.md)

## Internal Helpers

The module uses several utility functions:

- `dump_rval()` from [rval module](/python/assassyn/codegen/verilog/rval.md) for generating signal references
- `dump_type_cast()` from [utils module](/python/assassyn/codegen/verilog/utils.md) for type casting operations

The arithmetic expression generation is integrated into the main expression dispatch system through the [__init__.py](/python/assassyn/codegen/verilog/_expr/__init__.md) module, which routes different expression types to their appropriate code generation functions.

**Project-specific Knowledge Required**:
- Understanding of [expression dispatch system](/python/assassyn/codegen/verilog/_expr/__init__.md)
- Knowledge of [CIRCTDumper integration](/python/assassyn/codegen/verilog/design.md)
- Reference to [arithmetic expression types](/python/assassyn/ir/expr/arith.md)
