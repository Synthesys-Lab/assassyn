# Assassyn Implementation

The core Python implementation of Assassyn.
This includes:
- `ir/`: The intermediate representation (IR) data structure, including modules, expressions, and types,
  as well as their trace-based AST builder interfaces.
- `builder.py`: The AST builder utilities, including the singleton AST context, and the decorators.
- `experimental/`: Experimental features. Currently, it is the experimental functional frontend.
- `backend/`: The backend invoker utilities, including the helper functions to call the codegen, and
  the configuration of the backends.
- `codegen/': The code generation, including a Rust simulator backend, and a Verilog backend.