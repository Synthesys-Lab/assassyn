# Arithmetic IR Nodes

This module defines the Intermediate Representation node classes for arithmetic and logical operations. These classes represent binary and unary operations in the assassyn AST, providing type inference and code generation support for arithmetic expressions.

---

## Section 1. Exposed Interfaces

### class BinaryOp

The IR node class for all binary operations, taking a left-hand side (`lhs`) and right-hand side (`rhs`) operand.

#### Attributes

- `lhs: Value` - Left-hand side operand
- `rhs: Value` - Right-hand side operand

#### Static Constants

- `ADD = 200` - Addition operation
- `SUB = 201` - Subtraction operation  
- `MUL = 202` - Multiplication operation
- `DIV = 203` - Division operation
- `MOD = 204` - Modulo operation
- `BITWISE_AND = 206` - Bitwise AND operation
- `BITWISE_OR = 207` - Bitwise OR operation
- `BITWISE_XOR = 208` - Bitwise XOR operation
- `ILT = 209` - Integer less than comparison
- `IGT = 210` - Integer greater than comparison
- `ILE = 211` - Integer less than or equal comparison
- `IGE = 212` - Integer greater than or equal comparison
- `EQ = 213` - Equality comparison
- `SHL = 214` - Shift left operation
- `SHR = 215` - Shift right operation
- `NEQ = 216` - Not equal comparison

#### Methods

#### `__init__(self, opcode, lhs, rhs)`

```python
def __init__(self, opcode, lhs, rhs):
    assert isinstance(lhs, Value), f'{type(lhs)} is not a Value!'
    assert isinstance(rhs, Value), f'{type(rhs)} is not a Value!'
    super().__init__(opcode, [lhs, rhs])
```

**Explanation:** Initializes a binary operation node with the given opcode and operands. Validates that both operands are Value instances before storing them.

#### `lhs` (property)

```python
@property
def lhs(self):
    '''Get the left-hand side operand'''
    return self._operands[0]
```

**Explanation:** Returns the left-hand side operand of the binary operation.

#### `rhs` (property)

```python
@property
def rhs(self):
    '''Get the right-hand side operand'''
    return self._operands[1]
```

**Explanation:** Returns the right-hand side operand of the binary operation.

#### `dtype` (property)

```python
@property
def dtype(self):
    '''Get the data type of this operation'''
    # pylint: disable=import-outside-toplevel
    from ..dtype import Bits
    if self.opcode in [BinaryOp.ADD]:
        # TODO(@were): Make this bits + 1
        bits = max(self.lhs.dtype.bits, self.rhs.dtype.bits)
        tyclass = self.lhs.dtype.__class__
        return tyclass(bits)
    if self.opcode in [BinaryOp.SUB, BinaryOp.DIV, BinaryOp.MOD]:
        return type(self.lhs.dtype)(self.lhs.dtype.bits)
    if self.opcode in [BinaryOp.MUL]:
        bits = self.lhs.dtype.bits + self.rhs.dtype.bits
        tyclass = self.lhs.dtype.__class__
        return tyclass(bits)
    if self.opcode in [BinaryOp.SHL, BinaryOp.SHR]:
        return Bits(self.lhs.dtype.bits)
    if self.opcode in [BinaryOp.ILT, BinaryOp.IGT, BinaryOp.ILE, BinaryOp.IGE,
                       BinaryOp.EQ, BinaryOp.NEQ]:
        return Bits(1)
    if self.opcode in [BinaryOp.BITWISE_AND, BinaryOp.BITWISE_OR, BinaryOp.BITWISE_XOR]:
        return Bits(max(self.lhs.dtype.bits, self.rhs.dtype.bits))
    raise NotImplementedError(f'Unsupported binary operation {self.opcode}')
```

**Explanation:** Calculates and returns the data type of the operation result based on the operation type and operand types. The type inference rules are:
- Addition: Maximum bit width of operands (NOTE: Currently uses `max(bits)` but should be `bits + 1` for carry bit handling)
- Subtraction, Division, Modulo: Same type as left operand
- Multiplication: Sum of operand bit widths
- Shifts: Same bit width as left operand
- Comparisons: Single bit result
- Bitwise operations: Maximum bit width of operands

**Note on Addition Carry Handling:** The current implementation uses `max(self.lhs.dtype.bits, self.rhs.dtype.bits)` for addition operations, but there's a TODO comment indicating this should be `bits + 1` to account for carry bits. This is a known limitation that may be addressed in future versions.

#### `__repr__(self)`

```python
def __repr__(self):
    lval = self.as_operand()
    lhs = self.lhs.as_operand()
    rhs = self.rhs.as_operand()
    op = self.OPERATORS[self.opcode]
    return f'{lval} = {lhs} {op} {rhs}'
```

**Explanation:** Returns a human-readable string representation of the binary operation in the format `result = lhs op rhs`.

#### `is_computational(self)`

```python
def is_computational(self):
    '''Check if this operation is computational'''
    return self.opcode in [BinaryOp.ADD, BinaryOp.SUB, BinaryOp.MUL, BinaryOp.DIV,
                           BinaryOp.MOD]
```

**Explanation:** Returns True if the operation is a computational operation (arithmetic), False otherwise.

#### `is_comparative(self)`

```python
def is_comparative(self):
    '''Check if this operation is comparative'''
    return self.opcode in [BinaryOp.ILT, BinaryOp.IGT, BinaryOp.ILE, BinaryOp.IGE,
                           BinaryOp.EQ, BinaryOp.NEQ]
```

**Explanation:** Returns True if the operation is a comparison operation, False otherwise.

### class UnaryOp

The IR node class for unary operations with a single operand.

#### Static Constants

- `NEG = 100` - Negation operation
- `FLIP = 101` - Bitwise NOT operation

#### Methods

#### `__init__(self, opcode, x)`

```python
def __init__(self, opcode, x):
    super().__init__(opcode, [x])
```

**Explanation:** Initializes a unary operation node with the given opcode and operand.

#### `x` (property)

```python
@property
def x(self) -> Value:
    '''Get the operand of this unary operation'''
    return self._operands[0]
```

**Explanation:** Returns the operand of the unary operation.

#### `dtype` (property)

```python
@property
def dtype(self) -> DType:
    '''Get the data type of this unary operation'''
    # pylint: disable=import-outside-toplevel
    from ..dtype import Bits
    return Bits(self.x.dtype.bits)
```

**Explanation:** Returns the data type of the unary operation result, which has the same bit width as the operand.

#### `__repr__(self)`

```python
def __repr__(self):
    return f'{self.as_operand()} = {self.OPERATORS[self.opcode]}{self.x.as_operand()}'
```

**Explanation:** Returns a human-readable string representation of the unary operation in the format `result = op operand`.

---

## Section 2. Internal Helpers

This module contains no internal helper functions or data structures. All functionality is exposed through the BinaryOp and UnaryOp classes.
