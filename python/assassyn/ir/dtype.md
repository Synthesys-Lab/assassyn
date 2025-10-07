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
- **Example:**
  ```python
  i32 = Int(32)        # 32-bit signed integer type
  const_val = i32(-42) # Create constant -42 of type i32
  print(i32)           # Output: i32
  ```

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
- **Example:**
  ```python
  u16 = UInt(16)       # 16-bit unsigned integer type
  const_val = u16(255) # Create constant 255 of type u16
  print(u16)           # Output: u16
  ```

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
- **Example:**
  ```python
  b8 = Bits(8)         # 8-bit raw bits type
  const_val = b8(0xAB) # Create bit pattern 0xAB
  print(b8)            # Output: b8
  ```

-------

### `Float()` - Floating Point Type
```python
class Float(DType):
    def __init__(self)             # Create 32-bit floating point type
    def __repr__(self) -> str      # String representation as 'f32'
```
- **Description:** Represents 32-bit floating point numbers.
- **Bit Width:** Fixed at 32 bits
- **Example:**
  ```python
  f32 = Float()        # 32-bit floating point type
  print(f32)           # Output: f32
  ```

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
- **Example:**
  ```python
  void_type = void()   # Get void type
  ```

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
- **Example:**
  ```python
  arr_type = ArrayType(UInt(32), 16)  # Array of 16 32-bit unsigned integers
  print(arr_type.size)                # Output: 16
  print(arr_type.scalar_ty)           # Output: u32
  ```

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
- **Example:**
  ```python
  # Sequential layout (MSB to LSB order)
  packet = Record(header=UInt(16), payload=Bits(32), footer=UInt(8))
  
  # Explicit layout with bit positions
  control = Record({
      (31, 24): ("opcode", UInt(8)),
      (23, 16): ("src_reg", UInt(8)), 
      (15, 8):  ("dst_reg", UInt(8)),
      (7, 0):   ("flags", Bits(8))
  })
  ```

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
- **Example:**
  ```python
  # Create record type
  point = Record(x=Int(16), y=Int(16))
  
  # Create record value from field values
  origin = point.bundle(x=Int(16)(0), y=Int(16)(0))
  
  # Access fields
  x_coord = origin.x  # Extract x field
  y_coord = origin.y  # Extract y field
  ```

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
- **Example:**
  ```python
  const1 = to_uint(255)     # Creates u8(255) - minimal 8 bits
  const2 = to_uint(255, 16) # Creates u16(255) - explicit 16 bits
  ```

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
- **Example:**
  ```python
  const1 = to_int(-128)     # Creates appropriate signed type
  const2 = to_int(-128, 16) # Creates i16(-128) - explicit 16 bits
  ```

-------

## Usage Patterns

### Basic Type Creation
```python
# Create data types
u32 = UInt(32)           # 32-bit unsigned
i16 = Int(16)            # 16-bit signed  
raw_bits = Bits(8)       # 8-bit raw
float_type = Float()     # 32-bit float

# Create constants
val1 = u32(42)           # Unsigned constant
val2 = i16(-100)         # Signed constant
val3 = raw_bits(0xFF)    # Raw bit pattern
```

### Record/Struct Usage
```python
# Define CPU instruction format
instruction = Record(
    opcode=UInt(8),      # 8-bit operation code
    rs1=UInt(5),         # 5-bit source register 1
    rs2=UInt(5),         # 5-bit source register 2  
    rd=UInt(5),          # 5-bit destination register
    immediate=Int(9)     # 9-bit signed immediate
)

# Create instruction instance
add_inst = instruction.bundle(
    opcode=UInt(8)(0x10),    # ADD opcode
    rs1=UInt(5)(1),          # Register 1
    rs2=UInt(5)(2),          # Register 2
    rd=UInt(5)(3),           # Destination register 3
    immediate=Int(9)(0)      # No immediate
)

# Access instruction fields
op = add_inst.opcode         # Extract opcode field
src1 = add_inst.rs1          # Extract source register 1
```

### Array Type Usage
```python
# Create array types
memory_type = ArrayType(UInt(32), 1024)  # 1K words of 32-bit data
buffer_type = ArrayType(Bits(8), 256)    # 256-byte buffer

# Use in register arrays (see array.py)
cache_line = RegArray(memory_type.scalar_ty, memory_type.size)
```

### Type Checking and Validation
```python
# Type property checks
assert u32.is_int()          # True - UInt is integer type
assert not raw_bits.is_int() # False - Bits is not integer type
assert i16.is_signed()       # True - Int is signed
assert not u32.is_signed()   # False - UInt is unsigned

# Range validation
assert u32.inrange(100)      # True - 100 fits in u32
assert not u32.inrange(-1)   # False - negative doesn't fit in unsigned
assert i16.inrange(-32768)   # True - fits in 16-bit signed range
```

## Design Notes

### Field Ordering in Records
When using keyword arguments (`**kwargs`) to define record fields, Python 3.6+ guarantees that field order matches the argument order. Fields are laid out from MSB to LSB in the order provided:

```python
# Fields ordered as: header (MSB) -> payload -> footer (LSB)
packet = Record(header=UInt(8), payload=UInt(16), footer=UInt(8))
```

### Record Readonly Property
Records become readonly when using explicit bit layout with gaps:

```python
# This record has gaps and becomes readonly
sparse_record = Record({
    (31, 24): ("high_byte", UInt(8)),
    (15, 8):  ("mid_byte", UInt(8)),
    # Bits 23-16 and 7-0 are unassigned gaps
})

assert sparse_record.readonly  # True - has unassigned bits
```

### Type Bit Width Calculation
All types automatically calculate their bit width:
- **Primitive types:** Specified directly (`Int(32)` = 32 bits)
- **Array types:** `element_bits * array_size`  
- **Record types:** Sum of all field bits (sequential) or max bit position + 1 (explicit)

The type system ensures hardware realizability by tracking exact bit requirements for all composite structures.