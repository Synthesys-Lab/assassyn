# Data Type Module
The `dtype.py` module defines the data type system for the Assassyn IR, providing a comprehensive hierarchy of types for hardware description and verification. The module includes base types, integer types, raw bits, floating point, arrays, records, and utility functions.

```python
class DType:
    _bits: int  # Number of bits in this data type

class Int(DType):         # Signed integer data type
class UInt(DType):        # Unsigned integer data type  
class Bits(DType):        # Raw bits data type
class Float(DType):       # Floating point data type
class Void(DType):        # Void data type
class ArrayType(DType):   # Array data type
class Record(DType):      # Record/struct data type
class RecordValue:        # Value wrapper for record instances
```

## Data Type Hierarchy
The module provides a rich type system supporting hardware design patterns:
- **Base Types:** `DType` - foundation for all data types
- **Integer Types:** `Int`, `UInt` - signed and unsigned integers with configurable bit widths
- **Raw Types:** `Bits` - uninterpreted bit vectors
- **Composite Types:** `Record`, `ArrayType` - structured data types
- **Special Types:** `Void`, `Float` - utility and floating point types

## Exposed Interface
The `dtype.py` module provides data type classes, utility functions, and record value management for the Assassyn type system.

### `DType` - Base Data Type Class
```python
class DType:
    def __init__(self, bits: int)  # Initialize with bit width
    @property
    def bits(self) -> int          # Get number of bits in this data type
    def __eq__(self, other) -> bool # Check if two data types are equal
    def attributize(self, value, name)  # Create port syntax sugar
    def inrange(self, value) -> bool    # Check if value is in type range
    def is_int(self) -> bool           # Check if this is an integer type
    def is_raw(self) -> bool           # Check if this is raw bits type  
    def is_signed(self) -> bool        # Check if this is signed type
```
- **Description:** Base class for all data types in the Assassyn type system.
- **Properties:** 
  - `bits`: The bit width of the data type
- **Methods:** Type checking, comparison, and range validation utilities.

-------

### `Int(bits)` - Signed Integer Type
```python
class Int(DType):
    def __init__(self, bits: int)      # Create signed integer type
    def __call__(self, value: int)     # Create constant of this type
    def inrange(self, value) -> bool   # Check if value fits in signed range
    def __repr__(self) -> str          # String representation as 'i{bits}'
```
- **Description:** Represents signed integer types with configurable bit widths.
- **Parameters:**
  - `bits`: Number of bits for the integer (must be positive)
- **Range:** `-2^(bits-1)` to `2^(bits-1) - 1`

-------

### `UInt(bits)` - Unsigned Integer Type
```python
class UInt(DType):
    def __init__(self, bits: int)      # Create unsigned integer type (minimum 1 bit)
    def __call__(self, value: int)     # Create constant of this type
    def inrange(self, value) -> bool   # Check if value fits in unsigned range
    def __repr__(self) -> str          # String representation as 'u{bits}'
```
- **Description:** Represents unsigned integer types with configurable bit widths.
- **Parameters:**
  - `bits`: Number of bits for the integer (automatically clamped to minimum 1)
- **Range:** `0` to `2^bits - 1`

-------

### `Bits(bits)` - Raw Bits Type
```python
class Bits(DType):
    def __init__(self, bits: int)      # Create raw bits type
    def __call__(self, value: int)     # Create constant of this type
    def inrange(self, value) -> bool   # Check if value fits in bit range
    def __repr__(self) -> str          # String representation as 'b{bits}'
```
- **Description:** Represents raw bit vectors without arithmetic interpretation.
- **Parameters:**
  - `bits`: Number of bits in the vector
- **Range:** `0` to `2^bits - 1`

-------

### `Float()` - Floating Point Type
```python
class Float(DType):
    def __init__(self)             # Create 32-bit floating point type
    def __repr__(self) -> str      # String representation as 'f32'
```
- **Description:** Represents 32-bit floating point numbers.
- **Bit Width:** Fixed at 32 bits

-------

### `Void()` - Void Type
```python
class Void(DType):
    def __init__(self)             # Create void type (1 bit)
    def inrange(self, value) -> bool # Always returns False
```
- **Description:** Represents void/unit type for functions with no return value.
- **Bit Width:** 1 bit (minimal representation)
- **Range:** No valid values (inrange always returns False)

### `void()` - Void Type Factory
```python
def void() -> Void  # Create singleton void type instance
```
- **Description:** Factory function returning the singleton void type instance.
- **Returns:** The global `_VOID` instance.

-------

### `ArrayType(dtype, size)` - Array Type
```python
class ArrayType(DType):
    def __init__(self, dtype: DType, size: int)  # Create array type
    @property
    def size(self) -> int                        # Get array size
    @property  
    def scalar_ty(self) -> DType                 # Get element data type
```
- **Description:** Represents arrays of homogeneous elements with fixed size.
- **Parameters:**
  - `dtype`: Data type of array elements
  - `size`: Number of elements in the array
- **Bit Width:** `size * dtype.bits`

-------

### `Record(*args, **kwargs)` - Record/Struct Type
```python
class Record(DType):
    fields: dict     # Mapping of field names to (dtype, slice) tuples
    readonly: bool   # Whether record has gaps (readonly)
    
    def __init__(self, *args, **kwargs)           # Create record type
    def bundle(self, **kwargs) -> RecordValue     # Create record value
    def view(self, value) -> RecordValue          # Create record view of value
    def attributize(self, value, name)            # Extract field from record
    def __repr__(self) -> str                     # String representation
```
- **Description:** Represents structured data types (records/structs) with named fields.
- **Construction Modes:**
  1. **Explicit Layout:** `Record({(start, end): (name, dtype), ...})`
  2. **Sequential Layout:** `Record(field1=dtype1, field2=dtype2, ...)`
- **Properties:**
  - `fields`: Dictionary mapping field names to (dtype, bit_slice) tuples  
  - `readonly`: True if record has unassigned bit ranges

-------

### `RecordValue(dtype, *args, **kwargs)` - Record Value Wrapper
```python
class RecordValue:
    _payload: Value   # Underlying value of the record
    _dtype: Record    # Record type of this value
    
    def __init__(self, dtype: Record, *args, **kwargs)  # Create record value
    def value(self) -> Value                            # Get underlying value
    def as_operand(self)                               # Get as operand  
    @property
    def dtype(self) -> Record                          # Get record type
    def __getattr__(self, name)                        # Access record fields
```
- **Description:** Value wrapper providing field access for record instances.
- **Construction:**
  - From existing value: `RecordValue(record_type, existing_value)`
  - From field values: `RecordValue(record_type, field1=val1, field2=val2)`
- **Field Access:** Fields accessible as attributes via `record_value.field_name`

-------

### Utility Functions

### `to_uint(value, bits=None)` - Integer to UInt Conversion
```python
def to_uint(value: int, bits=None) -> Value  # Convert integer to UInt constant
```
- **Description:** Converts integer to unsigned integer constant with minimized bit width.
- **Parameters:**
  - `value`: Integer value to convert
  - `bits`: Optional bit width (defaults to minimal bits needed)
- **Returns:** UInt constant value

-------

### `to_int(value, bits=None)` - Integer to Int Conversion  
```python
def to_int(value: int, bits=None) -> Value   # Convert integer to Int constant
```
- **Description:** Converts integer to signed integer constant with minimized bit width.
- **Parameters:**
  - `value`: Integer value to convert  
  - `bits`: Optional bit width (defaults to minimal bits needed)
- **Returns:** Int constant value

-------

## Design Notes

### Field Ordering in Records
When using keyword arguments (`**kwargs`) to define record fields, Python 3.6+ guarantees that field order matches the argument order. Fields are laid out from MSB to LSB in the order provided.

### Record Readonly Property
Records become readonly when using explicit bit layout with gaps in the bit assignments.

### Type Bit Width Calculation
All types automatically calculate their bit width:
- **Primitive types:** Specified directly (`Int(32)` = 32 bits)
- **Array types:** `element_bits * array_size`  
- **Record types:** Sum of all field bits (sequential) or max bit position + 1 (explicit)

The type system ensures hardware realizability by tracking exact bit requirements for all composite structures.

### Equality and hashing
Types compare by class and bit width. A `Bits(8)` equals another `Bits(8)`, but not `UInt(8)`. Hashing follows the same rule so instances can serve as keys in caches.