# Constant Value IR Node

This file defines the `Const` class, the Abstract Syntax Tree (AST) node for representing constant literal values in the Assassyn IR system.

---

## Section 0. Summary

The `Const` class represents compile-time constant values in the IR. Unlike runtime expressions, constants are evaluated immediately during IR construction, providing optimization opportunities through constant folding and memoization. The class inherits from `Value` to participate in the operator overloading system while providing specialized implementations for bit slicing and concatenation operations.

---

## Section 1. Exposed Interfaces

### `Const`

```python
class Const(Value):
    '''
    The AST node data structure for constant values.
    
    Represents a compile-time constant with a specific data type and integer value.
    Provides specialized implementations for bit slicing and concatenation operations
    that are evaluated immediately rather than generating runtime expressions.
    '''
```

#### `__init__`

```python
def __init__(self, dtype: DType, value: int):
    '''
    Initialize a constant with the given data type and value.
    
    @param dtype The data type of this constant
    @param value The actual integer value of this constant
    '''
```

**Explanation:** Creates a new constant node after validating that the value fits within the range of the specified data type. The validation uses `dtype.inrange(value)` to ensure the value can be represented by the given data type.

#### `__repr__`

```python
def __repr__(self) -> str:
    '''
    Return a string representation of the constant.
    
    @return String in format "(value:dtype)"
    '''
```

**Explanation:** Provides a human-readable representation of the constant showing both its value and data type. This is used for debugging and IR visualization.

#### `as_operand`

```python
def as_operand(self) -> str:
    '''
    Dump the constant as an operand.
    
    @return String representation suitable for use as an operand in expressions
    '''
```

**Explanation:** Returns the same representation as `__repr__`. This method is part of the operand interface used throughout the IR system for generating code and debugging output.

#### `__getitem__`

```python
def __getitem__(self, x: slice) -> Const:
    '''
    Override the value slicing operation.
    
    @param x Slice object with start and stop indices
    @return New Const object representing the extracted bits
    '''
```

**Explanation:** Provides specialized bit slicing for constants that is evaluated immediately during IR construction. The operation extracts bits from `x.start` to `x.stop` (inclusive) and returns a new `Const` object with the appropriate smaller data type. This differs from the parent class `Value.__getitem__` which creates a `Slice` expression node.

The implementation:
1. Calculates the number of bits needed: `bits = x.stop - x.start + 1`
2. Validates the bit count is within supported range (≤ 32 bits)
3. Validates the source constant has enough bits
4. Creates a new `Bits(bits)` data type
5. Extracts the bits using bit manipulation: `(self.value >> x.start) & ((1 << bits) - 1)`
6. Returns a new constant via `_const_impl`

#### `concat`

```python
def concat(self, other: Value) -> Union[Const, Concat]:
    '''
    Concatenate two values together.
    
    @param other The value to concatenate with this constant
    @return New Const if other is also a Const, otherwise Concat expression
    '''
```

**Explanation:** Provides specialized concatenation for constants that is evaluated immediately when both operands are constants. This enables constant folding during IR construction. The implementation:

1. Checks if `other` is also a `Const`
2. If so, performs immediate concatenation:
   - Calculates the shift amount as `other.dtype.bits`
   - Creates a new `Bits(shift + self.dtype.bits)` data type
   - Computes the concatenated value: `(self.value << shift) | other.value`
   - Returns a new constant via `_const_impl`
3. If not, falls back to the parent class implementation which creates a `Concat` expression node

---

## Section 2. Internal Helpers

### `_const_impl`

```python
def _const_impl(dtype: DType, value: int) -> Const:
    '''
    The syntax sugar for creating a constant.
    
    @param dtype The data type for the constant
    @param value The integer value for the constant
    @return Const object, potentially from cache
    '''
```

**Explanation:** Internal helper function that implements constant creation with memoization. This function is called by data type constructors (e.g., `UInt(8)(42)`) to create constants efficiently.

The implementation:
1. Attempts to access the current builder singleton from the global context
2. If a builder is active, checks for an existing constant cache
3. Creates the cache if it doesn't exist
4. Uses `(dtype, value)` as the cache key
5. Returns cached constant if available
6. Otherwise creates a new `Const(dtype, value)` and caches it
7. Returns the constant (cached or newly created)

This memoization reduces memory usage and enables constant identity comparisons during IR construction. The cache is scoped to the builder context and is automatically cleared when the builder scope exits.
