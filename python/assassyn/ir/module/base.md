# Base Module

This file provides the foundational components for defining hardware modules, including a base class and a decorator for combinational logic.

-----

## Exposed Interfaces

```python
class ModuleBase:
    def as_operand(self) -> str
    def triggered(self) -> PureIntrinsic
    def add_external(self, operand: Operand) -> None

def combinational_for(module_type) -> Callable:
    ...
```

-----

## ModuleBase Class

The `ModuleBase` class is the parent class for all module definitions, providing core functionality for dependency tracking and introspection.

  * **External Dependency Tracking**: The class automatically tracks when the module's logic uses signals from other modules or arrays. The `add_external` method records these dependencies, which is crucial for connecting the system graph.
  * **Operand Representation**: The `as_operand` method provides a unique string identifier for the module, used when it is referenced as an input in an expression.
  * **Triggered Status**: The `triggered()` method is a frontend API that creates an intrinsic to check if the module was active in the current simulation cycle.

-----

## @combinational\_for Decorator

This is a powerful decorator applied to the function that defines a module's combinational logic.

  * **IR Context Management**: It automatically handles entering and exiting the module's context in the global IR builder. This ensures that all logic created within the decorated function is correctly associated with that module.
  * **Automatic Naming (AST Rewriting)**: Its key feature is the ability to parse the decorated function's Python source code and rewrite assignment statements. This allows the system to automatically infer meaningful signal names from the original variable names used in the Python code, leading to more debuggable output.
  * **Error Handling**: If the source code rewriting process fails, it prints a warning and gracefully falls back to using the original, un-modified function to ensure robustness.
