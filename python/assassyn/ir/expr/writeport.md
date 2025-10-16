# Array Write Port Module

This module enables multi-ported register array access by defining classes that support the syntactic sugar `(array & module)[index] <= value` for array writes. This allows multiple modules to write to the same array while maintaining proper hardware semantics.

## Design Documents

- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system and multi-port write support
- [Type System Design](../../../docs/design/lang/type.md) - Type system architecture and data type definitions
- [Memory System Architecture](../../../docs/design/arch/memory.md) - Memory system design

## Related Modules

- [Expression Base](../expr.md) - Base expression classes and operand system
- [Array Operations](../array.md) - Array read/write operations
- [Arithmetic Operations](../arith.md) - Arithmetic and logical operations
- [Intrinsic Operations](../intrinsic.md) - Intrinsic function operations

---

## Exposed Interfaces

### Core Classes

#### `class WritePort`

An object that represents a dedicated write connection from a specific module to an array. It is the entry point for the syntactic sugar and contains the core logic for creating the `ArrayWrite` node.

**Fields:**
- `array: Array` - The register array to write to
- `module: ModuleBase` - The module that owns this write port

**Methods:**
- `__init__(array: Array, module: ModuleBase)` - Initialize a WritePort
- `__getitem__(index)` - Return a proxy object that will handle the <= assignment
- `__setitem__(index, value)` - Handles the `(a&self)[0] = v` syntax directly
- `_create_write(index, value)` - Create an ArrayWrite operation with module information
- `__repr__()` - String representation of the WritePort

**Explanation:**
The `WritePort` class enables the `(array & module)` syntax by overloading the bitwise AND operator. When created, it registers itself with the array's `_write_ports` dictionary, ensuring each module has a unique write port to the array. This allows multiple modules to write to the same array without conflicts.

#### `class IndexedWritePort`

A temporary proxy object whose sole purpose is to capture the array `index` and handle the final `<=` assignment in the syntactic sugar chain.

**Fields:**
- `write_port: WritePort` - The parent WritePort object
- `index: typing.Union[int, Value]` - The array index

**Methods:**
- `__init__(write_port, index)` - Initialize the indexed write port
- `__le__(value)` - Overload <= operator for non-blocking assignment syntax

**Explanation:**
The `IndexedWritePort` class is returned by `WritePort.__getitem__()` and serves as an intermediate object in the syntactic sugar chain. It captures the index and provides the `<=` operator overload that triggers the actual `ArrayWrite` creation.

---

## Internal Helpers

### Syntactic Sugar Processing

The expression `(array & module)[index] <= value` is processed through the following steps:

1. **`(array & module)`**: The bitwise AND operator `&` is overloaded in the `Array` class to create a `WritePort` instance. This object represents a dedicated connection, binding the target `array` to the specific `module` that is performing the write.

2. **`[index]`**: The indexing operation `[]` is called on the `WritePort` object. This does not perform a write, but instead returns a temporary `IndexedWritePort` proxy object that stores both the parent `WritePort` and the `index`.

3. **`<= value`**: The less-than-or-equal operator `<=` is called on the `IndexedWritePort` proxy. This final step triggers the creation of the `ArrayWrite` IR node, passing the array, index, value, and the original module context to its constructor to correctly represent the multi-ported write in the IR.

### Write Port Management

Each `WritePort` instance is registered with its target array through the `_write_ports` dictionary. This ensures that:
- Each module has a unique write port to each array
- Multiple modules can write to the same array without conflicts
- The hardware semantics are preserved (non-blocking assignments)

### Type Validation

The `_create_write` method performs type validation to ensure:
- The index is either an integer or a `Value` object
- The value is either a `Value` or `RecordValue` object
- Proper type conversion for integer indices using `to_uint()`

### IR Builder Integration

The `_create_write` method uses the `@ir_builder` decorator to ensure that the `ArrayWrite` expression is properly inserted into the current IR block with correct source location information.

**Error Conditions:**
- Type validation errors: May occur if index is not an integer or `Value`, or if value is not a `Value` or `RecordValue`
- Write port conflicts: May occur if multiple modules attempt to write to the same array without proper write port management
- IR builder errors: May occur if the `@ir_builder` decorator fails to insert the expression into the current IR block
