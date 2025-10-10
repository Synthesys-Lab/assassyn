# Constant Value IR Node

This file defines the `Const` class, the Abstract Syntax Tree (AST) node for representing constant literal values.

-----

## Exposed Interfaces

```python
class Const(Value): ...
```

-----

## Const Class

The `Const` class is the IR node for a compile-time constant value, containing both the literal `value` and its associated data type (`DType`).

  * **Initialization**: A `Const` is created with a specific `DType` and an integer `value`. The constructor validates that the given value fits within the range of the specified data type.
  * **Bit Slicing (`__getitem__`)**: The class overrides the slicing operator to allow for direct bit extraction from the constant. This operation returns a new, smaller `Const` object representing the extracted bits.
  * **Constant Concatenation (`concat`)**: It provides a specialized implementation of the `.concat()` method for combining two `Const` objects. This operation is performed during IR construction, immediately computing the result and returning a new, larger `Const` object rather than creating a runtime `Concat` expression.

-----

## Construction and reuse

Constants are created through the data type call syntax (for example, `UInt(8)(42)`). When a system builder context is active, small constant nodes are memoized per builder to avoid duplicate allocations. This cache is keyed by `(dtype, value)` and is cleared when the builder scope exits.
