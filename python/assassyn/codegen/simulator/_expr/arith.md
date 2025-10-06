# Arithmetic Code Generation Module

This module provides helper functions to generate simulator code for arithmetic operations. It translates Intermediate Representation (IR) nodes for arithmetic into Rust code expressions for the simulator backend.

-----

## Exposed Interfaces

```python
def codegen_binary_op(node: BinaryOp, module_ctx, sys) -> str
def codegen_unary_op(node: UnaryOp, module_ctx, sys) -> str
```

-----

## Binary Operation Generation

The `codegen_binary_op` function generates a Rust expression for a `BinaryOp` IR node.

  * **Process**: The function maps the node's opcode to the corresponding Rust operator (e.g., `+`, `-`, `&`). It generates the code for the left and right-hand side operands and casts them to the appropriate Rust type.
  * **Generated Code Structure**: `(<lhs>) <op> (<rhs>)`
  * **Special Handling**: For signed right-shift operations (`SHR`), it explicitly casts operands to a signed integer type in Rust (like `i64`) to ensure an arithmetic, rather than logical, shift is performed.

-----

## Unary Operation Generation

The `codegen_unary_op` function generates a Rust expression for a `UnaryOp` IR node.

  * **Process**: It maps the node's opcode to a Rust operator (e.g., `FLIP` becomes `!`) and prepends it to the generated code for the operand.
  * **Generated Code Structure**: `<op>(<operand>)`
