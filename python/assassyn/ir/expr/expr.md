# Expression IR Nodes

This file provides the Abstract Syntax Tree data structures for expressions, including the foundational `Expr` and `Operand` classes.

-----

## Exposed Interfaces

```python
class Expr(Value): ...
class Operand: ...
class FIFOPop(Expr): ...
class Concat(Expr): ...
class Cast(Expr): ...
class Select(Expr): ...

# Frontend helper functions
def log(*args) -> Log: ...
def wire_assign(wire, value) -> WireAssign: ...
def wire_read(wire) -> WireRead: ...
```

-----

## Core Classes

  * **`Expr`**: The base class for all expression nodes in the IR. It serves as the foundation for operations and builds a use-def graph by tracking its inputs as `Operand`s.
  * **`Operand`**: A wrapper that creates a directed link between a value and the `Expr` that consumes it. This is the core mechanism for tracking dataflow dependencies.

-----

## Expression Node Types

The file defines numerous `Expr` subclasses to represent specific operations.

  * **`FIFOPop`**: Represents consuming a value from a port's FIFO; its resulting data type is derived from the port's type.
  * **`Concat`**: Represents the bit-concatenation of two values; the result's bit width is the sum of the operand widths.
  * **`Cast`**: Represents type conversions, including `bitcast`, zero-extend (`zext`), and sign-extend (`sext`) to a specified target type.
  * **`Select`**: Represents a ternary multiplexer that chooses between two values of the same type based on a condition.
  * **`Select1Hot`**: A specialized multiplexer controlled by a one-hot encoded signal.
  * **`WireRead` / `WireAssign`**: Represent reading from and assigning to a `Wire`, typically for external connections.
  * **`Log`**: A non-synthesizable node that functions as a print statement for debugging during simulation.

-----

## Frontend Functions

Helper functions like `log()`, `wire_read()`, and `wire_assign()` are provided to simplify IR construction. They use the `@ir_builder` decorator to automatically create the corresponding `Expr` node and insert it into the current IR block.
