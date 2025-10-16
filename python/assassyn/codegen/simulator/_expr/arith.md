# Arithmetic Code Generation Module

This module provides helper functions to generate simulator code for arithmetic operations. It translates Intermediate Representation (IR) nodes for arithmetic into Rust code expressions for the simulator backend.

## Summary

This module is part of the simulator code generation pipeline that converts Assassyn's IR arithmetic operations into executable Rust code. It handles both binary operations (addition, subtraction, comparison, bitwise operations) and unary operations (negation, bitwise flip) with proper type casting and special handling for signed arithmetic operations.

## Exposed Interfaces

### codegen_binary_op

```python
def codegen_binary_op(node: BinaryOp, module_ctx) -> str
```

Generates Rust code for binary arithmetic and logical operations.

**Parameters:**
- `node`: The BinaryOp IR node containing the operation to generate code for
- `module_ctx`: The module context containing module-specific information

**Returns:** A string containing the generated Rust expression

**Behavior:**
The function maps the node's opcode to the corresponding Rust operator (e.g., `+`, `-`, `&`, `==`) and generates code for both operands. It handles special cases for signed right-shift operations by explicitly casting operands to signed integer types to ensure arithmetic rather than logical shifts. The function also handles intrinsic operations in operands by delegating to the intrinsics codegen module.

**Generated Code Structure:** `ValueCastTo::<Type>::cast(&lhs) op ValueCastTo::<Type>::cast(&rhs)`

**Special Handling:**
- For signed right-shift (`SHR`) operations, operands are cast to signed types (`i32`, `i64`, or `BigInt`) to ensure arithmetic shift behavior
- Intrinsic operations in operands are handled by calling `codegen_intrinsic` from the intrinsics module
- Type casting uses `ValueCastTo` trait to ensure proper Rust type conversion

### codegen_unary_op

```python
def codegen_unary_op(node: UnaryOp, module_ctx) -> str
```

Generates Rust code for unary arithmetic and logical operations.

**Parameters:**
- `node`: The UnaryOp IR node containing the operation to generate code for
- `module_ctx`: The module context containing module-specific information

**Returns:** A string containing the generated Rust expression

**Behavior:**
The function maps the node's opcode to the corresponding Rust operator (e.g., `-` for negation, `!` for bitwise flip) and prepends it to the generated code for the operand. The operand is processed through `dump_rval_ref` to generate the appropriate reference.

**Generated Code Structure:** `op operand`

**Supported Operations:**
- `NEG`: Generates `-operand` for arithmetic negation
- `FLIP`: Generates `!operand` for bitwise complement

## Internal Helpers

This module does not contain internal helper functions. All functionality is exposed through the two main codegen functions.
