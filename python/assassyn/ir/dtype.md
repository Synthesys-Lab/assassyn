# Data Type Module

## Section 0. Summary

The `dtype.py` module implements the core type system for the Assassyn hardware description language. It provides a comprehensive hierarchy of data types that map directly to hardware representations, supporting both primitive types (integers, bits, floats) and composite types (arrays, records). The type system ensures hardware realizability by tracking exact bit requirements and providing range validation for all values.

The module is fundamental to the [trace-based DSL frontend](../../../docs/design/dsl.md), where all operations are implicitly typed and values must conform to their declared types. Types are used throughout the system for [module generation](../../../docs/design/module.md), [simulator code generation](../../../docs/design/simulator.md), and [Verilog code generation](../../../docs/design/pipeline.md).

## Section 1. Exposed Interfaces

### `DType` - Base Data Type Class

```python
class DType:
    def __init__(self, bits: int)
    @property
    def bits(self) -> int
    def __eq__(self, other) -> bool
    def attributize(self, value, name)
    def inrange(self, value) -> bool
    def is_int(self) -> bool
    def is_raw(self) -> bool
    def is_signed(self) -> bool
```

**Description:** Base class for all data types in the Assassyn type system. Provides common functionality for type checking, comparison, and range validation.

**Parameters:**
- `bits`: The bit width of the data type

**Properties:**
- `bits`: The number of bits in this data type

**Explanation:** The base class establishes the fundamental contract that all data types must have a known bit width. The `attributize` method is used by [Record types](#record-args-kwargs---recordstruct-type) to extract field values from composite data structures. The type checking methods (`is_int`, `is_raw`, `is_signed`) are used throughout the codebase for [arithmetic operations](../../expr/arith.md) and [code generation](../../codegen/simulator/utils.md) to determine appropriate handling of different data types.

-------

### `Int(bits)` - Signed Integer Type

```python
class Int(DType):
    def __init__(self, bits: int)
    def __call__(self, value: int)
    def inrange(self, value) -> bool
    def __repr__(self) -> str
```

**Description:** Represents signed integer types with configurable bit widths using two's complement representation.

**Parameters:**
- `bits`: Number of bits for the integer (must be positive)

**Range:** `-2^(bits-1)` to `2^(bits-1) - 1`

**Explanation:** Signed integers are fundamental to arithmetic operations in hardware. The `__call__` method creates [constant values](../const.md) of this type, which are used extensively in test cases like [test_async_call.py](../../../ci-tests/test_async_call.py) for creating immediate values. The `inrange` method validates that values fit within the signed range, which is crucial for [constant creation](../const.md) to prevent overflow errors.

-------

### `UInt(bits)` - Unsigned Integer Type

```python
class UInt(DType):
    def __init__(self, bits: int)
    def __call__(self, value: int)
    def inrange(self, value) -> bool
    def __repr__(self) -> str
```

**Description:** Represents unsigned integer types with configurable bit widths.

**Parameters:**
- `bits`: Number of bits for the integer (automatically clamped to minimum 1)

**Range:** `0` to `2^bits - 1`

**Explanation:** Unsigned integers are the most commonly used type in hardware design for counters, indices, and data values. The automatic clamping to minimum 1 bit ensures that even `UInt(0)` becomes `UInt(1)`, preventing invalid zero-width types. Used extensively in [array indexing](../array.md) and [register arrays](../array.md) throughout the system.

-------

### `Bits(bits)` - Raw Bits Type

```python
class Bits(DType):
    def __init__(self, bits: int)
    def __call__(self, value: int)
    def inrange(self, value) -> bool
    def __repr__(self) -> str
```

**Description:** Represents raw bit vectors without arithmetic interpretation.

**Parameters:**
- `bits`: Number of bits in the vector

**Range:** `0` to `2^bits - 1`

**Explanation:** Raw bits are used for uninterpreted data, bit manipulation, and when arithmetic operations are not needed. Commonly used in [record fields](#record-args-kwargs---recordstruct-type) and for bit-level operations. Unlike integer types, bits have no signed/unsigned semantics and are treated as pure bit vectors.

-------

### `Float()` - Floating Point Type

```python
class Float(DType):
    def __init__(self)
    def __repr__(self) -> str
```

**Description:** Represents 32-bit floating point numbers using IEEE 754 standard.

**Bit Width:** Fixed at 32 bits

**Explanation:** Currently limited to 32-bit floating point. Used for floating-point arithmetic operations when needed in hardware designs.

-------

### `Void()` - Void Type

```python
class Void(DType):
    def __init__(self)
    def inrange(self, value) -> bool
```

**Description:** Represents void/unit type for functions with no return value.

**Bit Width:** 1 bit (minimal representation)

**Range:** No valid values (inrange always returns False)

**Explanation:** Used for functions that don't return values, similar to `void` in C/C++. The 1-bit representation is minimal but allows the type to participate in the type system.

### `void()` - Void Type Factory

```python
def void() -> Void
```

**Description:** Factory function returning the singleton void type instance.

**Returns:** The global `_VOID` instance.

**Explanation:** Provides a convenient way to access the singleton void type without instantiating new objects.

-------

### `ArrayType(dtype, size)` - Array Type

```python
class ArrayType(DType):
    def __init__(self, dtype: DType, size: int)
    @property
    def size(self) -> int
    @property  
    def scalar_ty(self) -> DType
```

**Description:** Represents arrays of homogeneous elements with fixed size.

**Parameters:**
- `dtype`: Data type of array elements
- `size`: Number of elements in the array

**Bit Width:** `size * dtype.bits`

**Properties:**
- `size`: The number of elements in this array
- `scalar_ty`: The data type of the elements in this array

**Explanation:** Array types are used for [register arrays](../array.md) and memory structures. The total bit width is calculated as the product of element count and element bit width, ensuring hardware realizability.

-------

### `Record(*args, **kwargs)` - Record/Struct Type

```python
class Record(DType):
    fields: dict
    readonly: bool
    
    def __init__(self, *args, **kwargs)
    def bundle(self, **kwargs) -> RecordValue
    def view(self, value) -> RecordValue
    def attributize(self, value, name)
    def __repr__(self) -> str
```

**Description:** Represents structured data types (records/structs) with named fields and configurable bit layouts.

**Construction Modes:**
1. **Explicit Layout:** `Record({(start, end): (name, dtype), ...})` - Fields at specific bit positions
2. **Sequential Layout:** `Record(field1=dtype1, field2=dtype2, ...)` - Fields packed sequentially

**Properties:**
- `fields`: Dictionary mapping field names to (dtype, bit_slice) tuples  
- `readonly`: True if record has unassigned bit ranges (gaps in explicit layout)

**Explanation:** Records provide structured data organization similar to C structs or SystemVerilog structs. The explicit layout mode allows precise bit-level control for hardware interfaces, while sequential layout provides convenient field packing. The `readonly` property indicates whether the record has gaps in its bit layout, which affects whether new values can be created via `bundle()`. Used extensively in [test_record_large_bits.py](../../../ci-tests/test_record_large_bits.py) for complex data structures.

-------

### `RecordValue(dtype, *args, **kwargs)` - Record Value Wrapper

```python
class RecordValue:
    _payload: Value
    _dtype: Record
    
    def __init__(self, dtype: Record, *args, **kwargs)
    def value(self) -> Value
    def as_operand(self)
    @property
    def dtype(self) -> Record
    def __getattr__(self, name)
```

**Description:** Value wrapper providing field access for record instances.

**Construction:**
- From existing value: `RecordValue(record_type, existing_value)`
- From field values: `RecordValue(record_type, field1=val1, field2=val2)`

**Properties:**
- `_payload`: The underlying value of the record
- `_dtype`: The record type of this value

**Field Access:** Fields accessible as attributes via `record_value.field_name`

**Explanation:** RecordValue is a virtual wrapper that doesn't exist in the AST but provides convenient field access through Python's `__getattr__` mechanism. The actual AST nodes are the underlying `_payload` value. This design allows field access like `record.field_name` while maintaining the IR structure. Used in [array expressions](../expr/array.md) and [write port operations](../expr/writeport.md) for structured data manipulation.

-------

### Utility Functions

### `to_uint(value, bits=None)` - Integer to UInt Conversion

```python
def to_uint(value: int, bits=None) -> Value
```

**Description:** Converts integer to unsigned integer constant with minimized bit width.

**Parameters:**
- `value`: Integer value to convert
- `bits`: Optional bit width (defaults to minimal bits needed)

**Returns:** UInt constant value

**Explanation:** Automatically calculates the minimum bit width needed to represent the value using `value.bit_length()`. Used extensively in [array indexing](../array.md) to convert integer indices to proper UInt values for hardware operations.

-------

### `to_int(value, bits=None)` - Integer to Int Conversion  

```python
def to_int(value: int, bits=None) -> Value
```

**Description:** Converts integer to signed integer constant with minimized bit width.

**Parameters:**
- `value`: Integer value to convert  
- `bits`: Optional bit width (defaults to minimal bits needed)

**Returns:** Int constant value

**Explanation:** Similar to `to_uint` but creates signed integer constants. Used when signed arithmetic is required in hardware operations.

-------

## Section 2. Internal Helpers

### `_VOID` - Global Void Instance

```python
_VOID = Void()
```

**Description:** Singleton instance of the Void type used by the `void()` factory function.

**Explanation:** Maintains a single global instance to avoid creating multiple void type objects, following the singleton pattern for the void type.

-------

## Design Notes

### Field Ordering in Records
When using keyword arguments (`**kwargs`) to define record fields, Python 3.6+ guarantees that field order matches the argument order. Fields are laid out from MSB to LSB in the order provided, ensuring deterministic bit layout for hardware generation.

### Record Readonly Property
Records become readonly when using explicit bit layout with gaps in the bit assignments. This prevents creation of new record values via `bundle()` when the bit layout is incomplete, ensuring data integrity.

### Type Bit Width Calculation
All types automatically calculate their bit width:
- **Primitive types:** Specified directly (`Int(32)` = 32 bits)
- **Array types:** `element_bits * array_size`  
- **Record types:** Sum of all field bits (sequential) or max bit position + 1 (explicit)

The type system ensures hardware realizability by tracking exact bit requirements for all composite structures, which is essential for [Verilog code generation](../../codegen/verilog/) and [simulator generation](../../codegen/simulator/).

### Equality and Hashing
Types compare by class and bit width. A `Bits(8)` equals another `Bits(8)`, but not `UInt(8)`. Hashing follows the same rule so instances can serve as keys in caches, improving performance in type checking and code generation.

### RecordValue Virtual Wrapper Design
The `RecordValue` class is designed as a virtual wrapper that doesn't appear in the AST but provides convenient field access. This design decision allows Python-like field access (`record.field_name`) while maintaining the IR structure integrity. The actual AST nodes remain the underlying `_payload` value, ensuring proper code generation and analysis.