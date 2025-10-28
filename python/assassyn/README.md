# Assassyn Implementation

The core Python implementation of Assassyn.
This includes:
- `ir/`: The intermediate representation (IR) data structure, including modules, expressions, and types,
  as well as their trace-based AST builder interfaces.
- `builder.py`: The AST builder utilities, including the singleton AST context, and the decorators.
- `backend/`: The backend invoker utilities, including the helper functions to call the codegen, and
  the configuration of the backends.
- `codegen/`: The code generation, including a Rust simulator backend, and a Verilog backend.

Ideally, each `*.py` file shall have a corresponding `*.md` file that documents the exposed interface
and internal implementation details. It is prefered to read this markdown file to develop.

> NOTE: This is still working in progress, and the documentation is not complete yet.
