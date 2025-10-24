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

The `_create_write` method performs comprehensive type validation to ensure:
- The index is either an integer or a `Value` object
- The value is either a `Value` or `RecordValue` object
- Proper type conversion for integer indices using `to_uint()`
- **Strict type checking**: The value's type must exactly match the array's `scalar_ty` using `type_eq()` method
- **RecordValue handling**: RecordValue objects are automatically unwrapped to their underlying `Bits` representation before type checking and write creation
- **Record/Bits flexibility**: When array expects Record type and value is raw Bits (from `.value()` call), allows the write if bit widths match
- **Error reporting**: Type mismatches raise `TypeError` with detailed information about expected vs actual types

**Type Checking Process:**
1. Extract dtype from value (handling RecordValue uniformly)
2. For RecordValue: Unwrap to raw Bits immediately after extracting dtype
3. Perform unified type check that handles Record/Bits special case
4. For Record array with Bits value: Compare bit widths instead of using `type_eq()`
5. For all other cases: Use strict type checking using `type_eq()` against array's `scalar_ty`
6. Raise descriptive TypeError if types don't match

**Record/Bits Compatibility:**
When users explicitly call `.value()` on RecordValue to get raw Bits, the type checker allows this pattern if:
- Array expects Record type
- Value is raw Bits type  
- Bit widths match exactly (`value.dtype.bits == array.scalar_ty.bits`)

This follows the same pattern as `Bind._push` and enables the common frontend pattern where RecordValue is unwrapped for array writes. The simplified logic reduces code duplication and improves consistency across the codebase.

**Error Messages:**
Type mismatches produce detailed error messages in the format:
```
TypeError: Type mismatch in array write: array 'array_name' expects element type UInt(8), but got value of type UInt(16)
```
For Record/Bits width mismatches:
```
TypeError: Type mismatch in array write: array 'bundle' expects element type record { is_odd: b1, payload: b32 } (33 bits), but got value of type b65 (65 bits)
```

### IR Builder Integration

The `_create_write` method uses the `@ir_builder` decorator to ensure that the `ArrayWrite` expression is properly inserted into the current IR block with correct source location information.

**Error Conditions:**
- Type validation errors: May occur if index is not an integer or `Value`, or if value is not a `Value` or `RecordValue`
- Write port conflicts: May occur if multiple modules attempt to write to the same array without proper write port management
- IR builder errors: May occur if the `@ir_builder` decorator fails to insert the expression into the current IR block
