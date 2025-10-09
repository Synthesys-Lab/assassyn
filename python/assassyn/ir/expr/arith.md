# Arithmetic IR Nodes

This file defines the Intermediate Representation node classes for arithmetic and logical operations.

-----

## Exposed Interfaces

```python
class BinaryOp(Expr): ...
class UnaryOp(Expr): ...
```

-----

## BinaryOp Class

The `BinaryOp` class is the IR node for all binary operations, taking a left-hand side (`lhs`) and right-hand side (`rhs`) operand.

  * **Operations**: It represents a wide range of operations, including:
      * Arithmetic: `ADD`, `SUB`, `MUL`, `DIV`, `MOD`
      * Comparative: `ILT` (\<), `IGT` (\>), `EQ` (==), etc.
      * Bitwise: `BITWISE_AND`, `BITWISE_OR`, `BITWISE_XOR`
      * Shifts: `SHL`, `SHR`
  * **Result Type Calculation**: The class automatically determines the data type of the result based on the operation.
      * Multiplication (`MUL`) results in a bit width that is the sum of the operand widths.
      * Comparisons (`ILT`, `EQ`, etc.) always result in a single bit (`Bits(1)`).
      * Addition and bitwise operations typically result in a bit width equal to the maximum of the operand widths.
  * **String Representation**: The node has a human-readable text format.
      * **Example**: `_res = _a + _b`

-----

## UnaryOp Class

The `UnaryOp` class is the IR node for operations with a single operand.

  * **Operations**: It represents negation (`NEG`) and bitwise NOT (`FLIP`).
  * **Result Type Calculation**: The result of a unary operation has the same bit width as its operand.
  * **String Representation**: The node has a human-readable text format.
      * **Example**: `_res = !_a`
