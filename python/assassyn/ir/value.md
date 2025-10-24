# Value Module

## Section 0. Summary

The `Value` class is the foundational base class for all IR value types in Assassyn's trace-based DSL frontend. It enables Python operator overloading to automatically generate corresponding IR nodes, allowing natural Python syntax like `a + b` to create `BinaryOp` nodes. All IR value types (expressions, signals, ports) inherit from `Value` to gain automatic support for arithmetic, bitwise, comparison, and selection operations.

---

## Section 1. Exposed Interfaces

### class Value

Abstract base class for overloading arithmetic operations in the frontend. This class defines an abstract `dtype` property that must be implemented by all subclasses. All Value subclasses must provide a data type, even if it's `Void()` for side-effect operations.

#### `__add__`

```python
def __add__(self, other):
    '''
    Creates a binary addition operation.

    @param other The right operand for addition
    @return BinaryOp node with ADD opcode
    '''
```

**Explanation**: This method implements the `+` operator overloading. When Python evaluates `a + b` where `a` is a `Value`, it calls this method to create a `BinaryOp` node representing the addition operation. The method is decorated with `@ir_builder` to automatically inject the generated node into the current IR block.

#### `__sub__`

```python
def __sub__(self, other):
    '''
    Creates a binary subtraction operation.

    @param other The right operand for subtraction
    @return BinaryOp node with SUB opcode
    '''
```

**Explanation**: Implements the `-` operator overloading, creating a `BinaryOp` node with SUB opcode.

#### `__mul__`

```python
def __mul__(self, other):
    '''
    Creates a binary multiplication operation.

    @param other The right operand for multiplication
    @return BinaryOp node with MUL opcode
    '''
```

**Explanation**: Implements the `*` operator overloading, creating a `BinaryOp` node with MUL opcode.

#### `__or__`

```python
def __or__(self, other):
    '''
    Creates a bitwise OR operation.

    @param other The right operand for bitwise OR
    @return BinaryOp node with BITWISE_OR opcode
    '''
```

**Explanation**: Implements the `|` operator overloading, creating a `BinaryOp` node with BITWISE_OR opcode.

#### `__xor__`

```python
def __xor__(self, other):
    '''
    Creates a bitwise XOR operation.

    @param other The right operand for bitwise XOR
    @return BinaryOp node with BITWISE_XOR opcode
    '''
```

**Explanation**: Implements the `^` operator overloading, creating a `BinaryOp` node with BITWISE_XOR opcode.

#### `__and__`

```python
def __and__(self, other):
    '''
    Creates a bitwise AND operation.

    @param other The right operand for bitwise AND
    @return BinaryOp node with BITWISE_AND opcode
    '''
```

**Explanation**: Implements the `&` operator overloading, creating a `BinaryOp` node with BITWISE_AND opcode.

#### `__getitem__`

```python
def __getitem__(self, x):
    '''
    Creates a bit slice operation.

    @param x A slice object with start and stop values
    @return Slice node for bit extraction
    '''
```

**Explanation**: Enables bit extraction using slice syntax like `value[start:stop]`. Only slice objects are supported (not integer indexing). The slice must have explicit `start` and `stop` values. This creates a `Slice` node for bit extraction.

#### `__lt__`

```python
def __lt__(self, other):
    '''
    Creates a less-than comparison operation.

    @param other The right operand for comparison
    @return BinaryOp node with ILT opcode
    '''
```

**Explanation**: Implements the `<` operator overloading, creating a `BinaryOp` node with ILT (integer less than) opcode.

#### `__gt__`

```python
def __gt__(self, other):
    '''
    Creates a greater-than comparison operation.

    @param other The right operand for comparison
    @return BinaryOp node with IGT opcode
    '''
```

**Explanation**: Implements the `>` operator overloading, creating a `BinaryOp` node with IGT (integer greater than) opcode.

#### `__le__`

```python
def __le__(self, other):
    '''
    Creates a less-than-or-equal comparison operation.

    @param other The right operand for comparison
    @return BinaryOp node with ILE opcode
    '''
```

**Explanation**: Implements the `<=` operator overloading, creating a `BinaryOp` node with ILE (integer less than or equal) opcode.

#### `__ge__`

```python
def __ge__(self, other):
    '''
    Creates a greater-than-or-equal comparison operation.

    @param other The right operand for comparison
    @return BinaryOp node with IGE opcode
    '''
```

**Explanation**: Implements the `>=` operator overloading, creating a `BinaryOp` node with IGE (integer greater than or equal) opcode.

#### `__eq__`

```python
def __eq__(self, other):
    '''
    Creates an equality comparison operation.

    @param other The right operand for comparison
    @return BinaryOp node with EQ opcode
    '''
```

**Explanation**: Implements the `==` operator overloading, creating a `BinaryOp` node with EQ opcode.

#### `__ne__`

```python
def __ne__(self, other):
    '''
    Creates a not-equal comparison operation.

    @param other The right operand for comparison
    @return BinaryOp node with NEQ opcode
    '''
```

**Explanation**: Implements the `!=` operator overloading, creating a `BinaryOp` node with NEQ opcode.

#### `__mod__`

```python
def __mod__(self, other):
    '''
    Creates a modulo operation.

    @param other The right operand for modulo
    @return BinaryOp node with MOD opcode
    '''
```

**Explanation**: Implements the `%` operator overloading, creating a `BinaryOp` node with MOD opcode.

#### `__invert__`

```python
def __invert__(self):
    '''
    Creates a bitwise NOT operation.

    @return UnaryOp node with FLIP opcode
    '''
```

**Explanation**: Implements the `~` operator overloading, creating a `UnaryOp` node with FLIP opcode.

#### `__lshift__`

```python
def __lshift__(self, other):
    '''
    Creates a left shift operation.

    @param other The shift amount
    @return BinaryOp node with SHL opcode
    '''
```

**Explanation**: Implements the `<<` operator overloading, creating a `BinaryOp` node with SHL (shift left) opcode.

#### `__rshift__`

```python
def __rshift__(self, other):
    '''
    Creates a right shift operation.

    @param other The shift amount
    @return BinaryOp node with SHR opcode
    '''
```

**Explanation**: Implements the `>>` operator overloading, creating a `BinaryOp` node with SHR (shift right) opcode.

#### `__hash__`

```python
def __hash__(self):
    '''
    Returns the object identity hash for use as dictionary key.

    @return Integer hash based on object identity
    '''
```

**Explanation**: Returns `id(self)`, enabling `Value` instances to be used as dictionary keys based on object identity. This is essential for the `case()` method which uses `Value` objects as dictionary keys.

#### `optional`

```python
def optional(self, default, predicate=None):
    '''
    Creates an optional value with default fallback.

    @param default The default value to use when predicate is false
    @param predicate The condition to check (defaults to self.valid())
    @return Select node choosing between self and default
    '''
```

**Explanation**: Creates an optional value that selects between `self` and `default` based on a predicate. If `predicate` is `None`, uses `self.valid()` as the condition. This method is not decorated with `@ir_builder` because it internally calls `select()`, which already handles IR injection. Decorating it would cause duplicate node insertion.

#### `bitcast`

```python
def bitcast(self, dtype):
    '''
    Reinterprets bit representation as different type.

    @param dtype Target data type for bitcast
    @return Cast node with BITCAST opcode
    '''
```

**Explanation**: Reinterprets the bit representation as a different type without changing bits. Used for type punning between representations. Creates a `Cast` node with BITCAST opcode.

#### `zext`

```python
def zext(self, dtype):
    '''
    Zero-extends to wider type.

    @param dtype Target data type for zero extension
    @return Cast node with ZEXT opcode
    '''
```

**Explanation**: Zero-extends to a wider type by padding with zeros. Used for unsigned integer widening. Creates a `Cast` node with ZEXT opcode.

#### `sext`

```python
def sext(self, dtype):
    '''
    Sign-extends to wider type.

    @param dtype Target data type for sign extension
    @return Cast node with SEXT opcode
    '''
```

**Explanation**: Sign-extends to a wider type by replicating the sign bit. Used for signed integer widening. Creates a `Cast` node with SEXT opcode.

#### `concat`

```python
def concat(self, other):
    '''
    Concatenates two bit vectors.

    @param other The bit vector to concatenate
    @return Concat node with self in upper bits, other in lower bits
    '''
```

**Explanation**: Concatenates two bit vectors, creating a `Concat` node. The result places `self` in the upper bits and `other` in the lower bits.

#### `select`

```python
def select(self, true_value, false_value):
    '''
    Creates ternary selection operation.

    @param true_value Value returned when self is true
    @param false_value Value returned when self is false
    @return Select node implementing ternary selection
    '''
```

**Explanation**: Implements ternary selection, creating a `Select` node. Returns `true_value` if `self` evaluates to true, otherwise `false_value`. Equivalent to `self ? true_value : false_value` in C.

#### `case`

```python
def case(self, cases: dict['Value', 'Value']):
    '''
    Creates multi-way selection from dictionary.

    @param cases Dictionary mapping Value keys to Value results, with None as default
    @return Value selected based on matching case
    '''
```

**Explanation**: Implements multi-way selection from a dictionary mapping `Value` keys to `Value` results. The `None` key is required as the default case. Internally generates nested `select()` operations. This method is not decorated with `@ir_builder` because it internally calls `select()`, which already handles IR injection.

#### `select1hot`

```python
def select1hot(self, *args):
    '''
    Creates one-hot selection operation.

    @param args Variable number of values to select from
    @return Select1Hot node for one-hot selection
    '''
```

**Explanation**: Performs one-hot selection, creating a `Select1Hot` node. `self` is a one-hot encoded selector, and `args` are the values to select from.

#### `valid`

```python
def valid(self):
    '''
    Checks if this value is valid.

    @return PureIntrinsic node with VALUE_VALID opcode
    '''
```

**Explanation**: Creates a `PureIntrinsic` node with opcode VALUE_VALID to check if a value is valid. This operation is primarily meaningful in downstream modules for checking data flow validity. The `valid()` method is commonly used in conjunction with `optional()` to provide default values when data is not available.

#### `dtype` (abstract property)

```python
@property
@abstractmethod
def dtype(self):
    '''
    Abstract property for data type. All Value subclasses must implement this.
    
    @return DType The data type of this value
    '''
```

**Explanation**: Abstract property that must be implemented by all Value subclasses. This ensures that every value in the IR has a well-defined data type, eliminating the need for `hasattr(v, 'dtype')` checks throughout the codebase. Side-effect operations (like `Log`, `FIFOPush`, `Bind`, `AsyncCall`, `ArrayWrite`) return `Void()` type.
