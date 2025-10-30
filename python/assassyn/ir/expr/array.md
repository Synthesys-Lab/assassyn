# Array Operation IR Nodes

This module defines the Intermediate Representation node classes for array read and write operations. These classes represent array access operations in the assassyn AST, providing support for both reading from and writing to array elements with proper module context tracking.

## Design Documents

- [Type System Design](../../../docs/design/lang/type.md) - Type system architecture and data type definitions
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system and multi-port write support
- [Memory System Architecture](../../../docs/design/arch/memory.md) - Memory system design including SRAM and DRAM

## Related Modules

- [Expression Base](../expr.md) - Base expression classes and operand system
- [Arithmetic Operations](../arith.md) - Arithmetic and logical operations
- [Write Port Operations](../writeport.md) - Multi-port array write support
- [Intrinsic Operations](../intrinsic.md) - Intrinsic function operations

---

## Section 1. Exposed Interfaces

### class ArrayWrite

The IR node class for array write operations, representing `arr[idx] = val`.

#### Static Constants

- `ARRAY_WRITE = 401` - Array write operation opcode

#### Attributes

- `module: ModuleBase` - The module performing the write operation

#### Methods

#### `__init__(self, arr, idx: Value, val: Value, module: ModuleBase = None, meta_cond=None)`

```python
def __init__(self, arr, idx: Value, val: Value, module: ModuleBase = None, meta_cond=None):
    # Get module from Singleton if not provided
    if module is None:
        # pylint: disable=import-outside-toplevel
        from ...builder import Singleton
        module = Singleton.peek_builder().current_module
    super().__init__(ArrayWrite.ARRAY_WRITE, [arr, idx, val], meta_cond=meta_cond)
    self.module = module
```

**Explanation:** Initializes an array write operation with the target array, index, value, predicate, and module context. If no module is provided, it retrieves the current module from the builder singleton via `Singleton.peek_builder()`. Predicate metadata is captured by the base `Expr` constructor, which snapshots the active predicate stack (or uses the explicitly supplied `meta_cond`) so downstream consumers can reuse the same guards without recomputing condition stacks. This module and predicate context is crucial for [multi-port write support](../../../docs/design/pipeline.md) where multiple modules may write to the same array while remaining gated by guard conditions.

**Note on Builder Context Dependency:** The `ArrayWrite` class depends on `Singleton.peek_builder()` when no module is explicitly provided. Callers must ensure a builder is active or supply the module explicitly to avoid runtime errors.

**Error Conditions:**
- `AssertionError`: Raised if `arr` is not an `Array` instance or `idx` is not a `Value` instance during `ArrayRead` initialization
- `AssertionError`: Raised if `value` is not a `Value` or `RecordValue` instance during `__le__` operation
- Context dependency: `ArrayWrite` operations may fail if no builder context is available and no module is explicitly provided

#### `array` (property)

```python
@property
def array(self) -> Array:
    '''Get the array to write to'''
    return self._operands[0]
```

**Explanation:** Returns the target array for the write operation.

#### `idx` (property)

```python
@property
def idx(self) -> Value:
    '''Get the index to write at'''
    return self._operands[1]
```

**Explanation:** Returns the index where the value will be written.

#### `val` (property)

```python
@property
def val(self) -> Value:
    '''Get the value to write'''
    return self._operands[2]
```

**Explanation:** Returns the value to be written to the array.

#### `dtype` (property)

```python
@property
def dtype(self):
    '''Get the data type of this operation (Void for side-effect operations)'''
    from ..dtype import void
    return void()
```

**Explanation:** Returns `Void()` type since array write operations are side-effect operations that don't produce a value.

#### `__repr__(self)`

```python
def __repr__(self):
    module_info = f' /* {self.module.name} */' if self.module else ''
    meta = self.meta_cond
    if meta is None:
        meta_info = ''
    else:
        operand = meta.as_operand() if hasattr(meta, 'as_operand') else repr(meta)
        meta_info = f' // meta cond {operand}'
    return (
        f'{self.array.as_operand()}[{self.idx.as_operand()}]'
        f' <= {self.val.as_operand()}{module_info}{meta_info}'
    )
```

**Explanation:** Returns a human-readable string representation of the array write operation in the format `array[index] <= value /* module_name */ // meta cond`, including the module context and captured predicate metadata for debugging purposes.

### class ArrayRead

The IR node class for array read operations, representing the value of `arr[idx]`.

#### Static Constants

- `ARRAY_READ = 400` - Array read operation opcode

#### Methods

#### `__init__(self, arr: Array, idx: Value)`

```python
def __init__(self, arr: Array, idx: Value):
    # pylint: disable=import-outside-toplevel
    from ..array import Array
    assert isinstance(arr, Array), f'{type(arr)} is not an Array!'
    assert isinstance(idx, Value), f'{type(idx)} is not a Value!'
    super().__init__(ArrayRead.ARRAY_READ, [arr, idx])
```

**Explanation:** Initializes an array read operation with the source array and index. Validates that the array is an Array instance and the index is a Value instance.

#### `array` (property)

```python
@property
def array(self) -> Array:
    '''Get the array to read from'''
    return self._operands[0]
```

**Explanation:** Returns the source array for the read operation.

#### `idx` (property)

```python
@property
def idx(self) -> Value:
    '''Get the index to read at'''
    return self._operands[1]
```

**Explanation:** Returns the index to read from.

#### `dtype` (property)

```python
@property
def dtype(self) -> DType:
    '''Get the data type of the read value'''
    return self.array.scalar_ty
```

**Explanation:** Returns the data type of the read value, which is the same as the array's element type.

#### `__repr__(self)`

```python
def __repr__(self):
    return f'{self.as_operand()} = {self.array.as_operand()}[{self.idx.as_operand()}]'
```

**Explanation:** Returns a human-readable string representation of the array read operation in the format `result = array[index]`.

#### `__getattr__(self, name)`

```python
def __getattr__(self, name):
    return self.dtype.attributize(self, name)
```

**Explanation:** Delegates attribute access to the data type's `attributize` method, enabling field access for structured data types.

#### `__le__(self, value)`

```python
def __le__(self, value):
    '''
    Handle the <= operator for array writes.
    '''
    # pylint: disable=import-outside-toplevel
    from ...builder import Singleton
    from ..dtype import RecordValue

    assert isinstance(value, (Value, RecordValue)), \
        f"Value must be Value or RecordValue, got {type(value)}"

    current_module = Singleton.peek_builder().current_module

    write_port = self.array & current_module
    return write_port._create_write(self.idx.value, value)
```

**Explanation:** Implements the `<=` operator to provide syntactic sugar for array writes. When used as `array[index] <= value`, this method creates an `ArrayWrite` operation through the array's write port system. This enables the intuitive syntax while maintaining proper [multi-port write support](../../../docs/design/pipeline.md).

---

## Section 2. Internal Helpers

This module contains no internal helper functions or data structures. All functionality is exposed through the ArrayWrite and ArrayRead classes.
