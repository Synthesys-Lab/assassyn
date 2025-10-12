# Array Module

The `array.py` module defines the `Array` class for representing register arrays in the Assassyn IR, along with the `RegArray` function for declaring them. Register arrays are fundamental data structures that store multiple values of the same type, accessible via indexing operations.

## Summary

This module provides the core infrastructure for register arrays in Assassyn's IR. Register arrays are used extensively throughout the system for storing stateful data, implementing memory structures, and managing pipeline stage registers. The module supports both single-port and multi-port access patterns through the `WritePort` mechanism, enabling multiple modules to write to the same array while maintaining proper hardware semantics.

## Exposed Interfaces

The `array.py` module provides the `RegArray` function and `Array` class methods for creating and manipulating register arrays.

### `RegArray`

```python
def RegArray(
    scalar_ty: DType,
    size: int,
    initializer: list = None,
    name: str = None,
    attr: list = None,
) -> Array:
    '''
    The frontend API to declare a register array.

    @param scalar_ty The data type of the array elements.
    @param size The size of the array. MUST be a compilation time constant.
    @param initializer The initializer of the register array. If not set, it is 0-initialized.
    @param name The custom name for the array.
    @param attr The attribute list of the array.
    @return Array instance registered with the AST builder.
    '''
```

**Explanation:**

This function serves as the primary interface for creating register arrays in Assassyn. It creates an `Array` instance and automatically registers it with the global builder singleton for proper IR construction. The function handles naming semantics by integrating with the [naming manager](../builder/naming_manager.md) to provide meaningful names when no explicit name is given.

The naming behavior follows a hierarchical approach:
- If `name` is provided, it is sanitized using [namify](../../utils.md#namify) and applied directly
- If no explicit name is given and a module context is active, a semantic name is assigned using the module name as a prefix (e.g., `<module>_array`)
- Semantic names are stored on the instance and used by `as_operand()` and `__repr__` methods

**Example:**
```python
my_array = RegArray(UInt(32), 16, name="register_file")  # 16-element array of 32-bit uints
```

## Internal Helpers

### `Array` Class

```python
class Array:
    '''
    The class represents a register array in the AST IR.
    '''
    scalar_ty: DType  # Data type of each element in the array
    size: int  # Size of the array
    initializer: list  # Initial values for the array elements
    attr: list  # Attributes of the array
    _users: typing.List[Expr]  # Users of the array
    _name: str  # Internal name storage
    _write_ports: typing.Dict['ModuleBase', 'WritePort']  # Write ports for this array
```

#### `as_operand`

```python
def as_operand(self) -> str:
    '''
    Dump the array as an operand.

    @return String representing the array's name for use in IR expressions.
    '''
```

#### `name` Property

```python
@property
def name(self) -> str:
    '''
    The name of the array. If not set, a default name is generated.

    @return String name of the array.
    '''

@name.setter
def name(self, name: str):
    '''
    Set custom array name.

    @param name The name to set for the array.
    '''
```

**Explanation:**

The name property implements a hierarchical naming system that prioritizes semantic names over internal names. It first checks for a semantic name stored in `__assassyn_semantic_name__`, then falls back to the internal `_name` field, and finally generates a default name using [identifierize](../../utils.md#identifierize) if neither is available.

#### `users` Property

```python
@property
def users(self) -> typing.List[Expr]:
    '''
    Get the users of the array.

    @return List of expressions that reference the array.
    '''
```

#### `__and__`

```python
def __and__(self, other) -> WritePort | BinaryOp:
    '''
    Overload & operator to create WritePort when combined with a Module.
    This enables write access: (array & module)[idx] <= value

    @param other A ModuleBase or Value.
    @return WritePort for module access or BinaryOp for bitwise AND.
    '''
```

**Explanation:**

This method implements the multi-port write access pattern used throughout Assassyn. When combined with a `ModuleBase`, it creates or retrieves a `WritePort` that enables the syntactic sugar `(array & module)[index] <= value`. This pattern is essential for hardware design where multiple modules need to write to the same array while maintaining proper port semantics.

The method also supports fallback to regular bitwise AND operations when the operand is a `Value`, maintaining compatibility with standard Python operations.

#### `__repr__`

```python
def __repr__(self) -> str:
    '''
    Enhanced repr to show write port information.

    @return String representation of the array, including name, type, size, and write ports.
    '''
```

#### `index_bits` Property

```python
@property
def index_bits(self) -> int:
    '''
    Get the number of bits needed to index the array.

    @return Integer bit count required for indexing.
    '''
```

**Explanation:**

This property calculates the minimum number of bits needed to index all elements in the array. It includes an optimization for power-of-2 sized arrays, where one less bit is needed due to the binary representation.

#### `index_type`

```python
def index_type(self) -> UInt:
    '''
    Get the type of the index.

    @return UInt type for array indexing based on index_bits.
    '''
```

#### `get_write_ports`

```python
def get_write_ports(self) -> typing.Dict['ModuleBase', 'WritePort']:
    '''
    Get the write_ports.

    @return Dictionary mapping modules to WritePort objects.
    '''
```

#### `__getitem__`

```python
@ir_builder
def __getitem__(self, index: typing.Union[int, Value]) -> ArrayRead:
    '''
    Read from array at specified index.

    @param index Integer or Value for the array index.
    @return ArrayRead expression.
    '''
```

**Explanation:**

This method implements array read operations with caching to avoid duplicate reads within the same block. It uses the builder's `array_read_cache` to store and retrieve previously created `ArrayRead` expressions, improving IR efficiency.

The method automatically converts integer indices to `UInt` values using [to_uint](../dtype.md#to_uint) and creates `ArrayRead` expressions that represent the read operation in the IR.

#### `get_flattened_size`

```python
def get_flattened_size(self) -> int:
    '''
    Get the flattened size of the array.

    @return Total bit count of the array (size * scalar_ty.bits).
    '''
```

#### `__setitem__`

```python
@ir_builder
def __setitem__(self, index, value):
    '''
    Write to array at specified index using current module's write port.

    @param index Integer or Value for the array index.
    @param value Value or RecordValue to write.
    '''
```

**Explanation:**

This method implements array write operations by creating a write port for the current module and delegating to the write port's `_create_write` method. It automatically handles the conversion of integer indices to `Value` objects and ensures proper type checking for the value being written.

The write operation uses the `&` operator internally to create or retrieve the appropriate `WritePort` for the current module context.

### `Slice` Class

```python
class Slice(Expr):
    '''
    The class for slice operation, where x[l:r] as a right value.
    '''
    SLICE = 700  # Operation type constant
```

#### `__init__`

```python
def __init__(self, x, l: int, r: int):
    '''
    Initialize a slice operation.

    @param x The value to slice.
    @param l The left bound of the slice (must be int literal).
    @param r The right bound of the slice (must be int literal).
    '''
```

**Explanation:**

The `Slice` class represents bit-slicing operations in the IR, where `x[l:r]` extracts bits from position `l` to `r` (inclusive) from value `x`. This is commonly used in hardware design for extracting specific bit fields from wider values.

The class enforces that both slice bounds must be integer literals at compile time, as hardware bit-slicing requires constant indices. The bounds are automatically converted to `UInt` values using [to_uint](../dtype.md#to_uint).

#### `x` Property

```python
@property
def x(self) -> Value:
    '''
    Get the value to slice.

    @return The Value being sliced.
    '''
```

#### `l` Property

```python
@property
def l(self) -> int:
    '''
    Get the left bound of the slice.

    @return The left bound index.
    '''
```

#### `r` Property

```python
@property
def r(self) -> int:
    '''
    Get the right bound of the slice.

    @return The right bound index.
    '''
```

#### `dtype` Property

```python
@property
def dtype(self) -> DType:
    '''
    Get the data type of the sliced value.

    @return Bits type with width (r - l + 1).
    '''
```

**Explanation:**

This property calculates the resulting data type of the slice operation. It creates a `Bits` type with width equal to `r - l + 1`, representing the number of bits extracted by the slice operation.

#### `__repr__`

```python
def __repr__(self):
    '''
    String representation of the slice operation.

    @return Formatted string showing the slice operation.
    '''
```