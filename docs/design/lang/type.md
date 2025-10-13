# Data Types and Type System

This document describes the data types available in Assassyn and the result types produced by various operations. The type system ensures hardware realizability by tracking exact bit requirements for all values and operations.

For the conceptual overview of the DSL, see [dsl.md](./dsl.md).
For implementation details, see [trace.md](./trace.md).
For practical usage examples, see the [tutorials](../../../tutorials/) folder.

## Primitive Data Types

### Unsigned Integers: `UInt(bits)`

**Description**: Unsigned integer types with configurable bit widths.

**Range**: `0` to `2^bits - 1`

**Usage**:
```python
# 8-bit unsigned integer
counter = UInt(8)(42)

# 32-bit unsigned integer  
address = UInt(32)(0x1000)

# Automatic bit width calculation
value = UInt()(255)  # Creates UInt(8) automatically
```

**Properties**:
- Minimum 1 bit (automatically clamped)
- Most commonly used type for counters, indices, and data values
- Supports all arithmetic operations

### Signed Integers: `Int(bits)`

**Description**: Signed integer types using two's complement representation.

**Range**: `-2^(bits-1)` to `2^(bits-1) - 1`

**Usage**:
```python
# 16-bit signed integer
offset = Int(16)(-100)

# 32-bit signed integer
result = Int(32)(-2147483648)
```

**Properties**:
- Supports negative values
- Used when signed arithmetic is required
- Same arithmetic operations as unsigned integers

### Raw Bits: `Bits(bits)`

**Description**: Raw bit vectors without arithmetic interpretation.

**Range**: `0` to `2^bits - 1`

**Usage**:
```python
# 8-bit raw data
data = Bits(8)(0xAB)

# 1-bit flag
flag = Bits(1)(1)
```

**Properties**:
- No signed/unsigned semantics
- Used for uninterpreted data and bit manipulation
- Bitwise operations preserve bit width
- Commonly used in record fields

### Floating Point: `Float()`

**Description**: 32-bit floating point numbers using IEEE 754 standard.

**Bit Width**: Fixed at 32 bits

**Usage**:
```python
# Floating point value
pi = Float()(3.14159)
```

**Properties**:
- Currently limited to 32-bit precision
- Used for floating-point arithmetic operations

### Void: `void()`

**Description**: Void/unit type for functions with no return value.

**Bit Width**: 1 bit (minimal representation)

**Usage**:
```python
# Function that doesn't return a value
def no_return():
    return void()
```

**Properties**:
- No valid values (inrange always returns False)
- Used for functions that don't return values
- Similar to `void` in C/C++

## Composite Data Types

### Arrays: `ArrayType(dtype, size)`

**Description**: Arrays of homogeneous elements with fixed size.

**Bit Width**: `size * dtype.bits`

**Usage**:
```python
# Array of 8-bit unsigned integers
registers = ArrayType(UInt(8), 16)

# Array of 32-bit signed integers  
memory = ArrayType(Int(32), 1024)
```

**Properties**:
- Fixed size at creation time
- All elements have the same data type
- Used for register arrays and memory structures
- Total bit width calculated as product of element count and element bit width

### Records/Structs: `Record(*args, **kwargs)`

**Description**: Structured data types with named fields and configurable bit layouts.

**Construction Modes**:

1. **Sequential Layout** (most common):
```python
# Fields packed sequentially from MSB to LSB
Packet = Record(
    header=UInt(8),
    payload=UInt(16), 
    checksum=UInt(8)
)
```

2. **Explicit Layout** (for precise bit control):
```python
# Fields at specific bit positions
ControlReg = Record({
    (31, 24): ("opcode", UInt(8)),
    (23, 16): ("address", UInt(8)),
    (15, 0):  ("data", UInt(16))
})
```

**Properties**:
- Field order guaranteed in Python 3.6+ (sequential mode)
- Fields laid out from MSB to LSB in argument order
- `readonly` property indicates gaps in bit layout
- Similar to C structs or SystemVerilog structs

## Struct Syntax Sugar

### Creating Record Values

**From field values**:
```python
Packet = Record(header=UInt(8), payload=UInt(16))

# Create record value
packet = Packet.bundle(header=0x01, payload=0x1234)
```

**From existing value**:
```python
# Create view of existing value
raw_value = UInt(32)(0x01001234)
packet = Packet.view(raw_value)
```

### Field Access

**Attribute access**:
```python
# Access fields as attributes
header_val = packet.header
payload_val = packet.payload

# Fields are immutable (right-value objects)
# packet.header = 0x02  # This would not work
```

**Field properties**:
- Fields accessible as `record_value.field_name`
- Each field access returns a `Value` object
- Fields are immutable (read-only)
- Virtual wrapper design - doesn't exist in AST

## Operation Result Types

### Arithmetic Operations

**Addition (`+`)**:
- Result type: Same class as operands
- Bit width: `max(lhs.bits, rhs.bits)` (NOTE: Should be `bits + 1` for carry)
- Example: `UInt(8) + UInt(16) → UInt(16)`

**Subtraction (`-`)**:
- Result type: Same as left operand
- Bit width: Same as left operand
- Example: `UInt(16) - UInt(8) → UInt(16)`

**Multiplication (`*`)**:
- Result type: Same class as operands
- Bit width: `lhs.bits + rhs.bits`
- Example: `UInt(8) * UInt(8) → UInt(16)`

**Division (`/`) and Modulo (`%`)**:
- Result type: Same as left operand
- Bit width: Same as left operand
- Example: `UInt(16) / UInt(8) → UInt(16)`

### Bitwise Operations

**Bitwise AND (`&`), OR (`|`), XOR (`^`)**:
- Result type: `Bits`
- Bit width: `max(lhs.bits, rhs.bits)`
- Example: `UInt(8) & UInt(16) → Bits(16)`

**Shift Left (`<<`), Shift Right (`>>`)**:
- Result type: `Bits`
- Bit width: Same as left operand
- Example: `UInt(8) << 2 → Bits(8)`

### Comparison Operations

**Less Than (`<`), Greater Than (`>`), Less Equal (`<=`), Greater Equal (`>=`)**:
- Result type: `Bits(1)`
- Example: `UInt(8) < UInt(16) → Bits(1)`

**Equality (`==`), Not Equal (`!=`)**:
- Result type: `Bits(1)`
- Example: `UInt(8) == UInt(8) → Bits(1)`

### Unary Operations

**Negation (`-`)**:
- Result type: `Bits`
- Bit width: Same as operand
- Example: `-UInt(8) → Bits(8)`

**Bitwise NOT (`~`)**:
- Result type: `Bits`
- Bit width: Same as operand
- Example: `~UInt(8) → Bits(8)`

### Type Conversion Operations

**Bitcast**:
- Result type: Target type
- Bit width: Target type bit width
- Example: `UInt(8).bitcast(Int(8)) → Int(8)`

**Zero Extend (`zext`)**:
- Result type: Target type
- Bit width: Target type bit width
- Example: `UInt(8).zext(UInt(16)) → UInt(16)`

**Sign Extend (`sext`)**:
- Result type: Target type
- Bit width: Target type bit width
- Example: `Int(8).sext(Int(16)) → Int(16)`

### Concatenation

**Bit Concatenation (`concat`)**:
- Result type: `Bits`
- Bit width: Sum of operand widths
- Example: `concat(UInt(8), UInt(8)) → Bits(16)`

### Selection Operations

**Ternary Select (`select`)**:
- Result type: Same as true/false values
- Bit width: Same as true/false values
- Example: `select(condition, UInt(8), UInt(8)) → UInt(8)`

**One-Hot Select (`select1hot`)**:
- Result type: Same as selected value
- Bit width: Same as selected value
- Example: `select1hot(onehot_signal, [UInt(8), UInt(8)]) → UInt(8)`

## Type System Design Notes

### Hardware Realizability
All types automatically calculate their bit width to ensure hardware realizability:
- **Primitive types**: Specified directly (`Int(32)` = 32 bits)
- **Array types**: `element_bits * array_size`
- **Record types**: Sum of all field bits (sequential) or max bit position + 1 (explicit)

### Type Equality and Hashing
Types compare by class and bit width:
- `Bits(8)` equals another `Bits(8)`, but not `UInt(8)`
- Hashing follows the same rule for cache performance
- Used for type checking and code generation optimization

### RecordValue Virtual Wrapper
The `RecordValue` class is designed as a virtual wrapper that doesn't appear in the AST but provides convenient field access through Python's `__getattr__` mechanism. This allows Python-like field access (`record.field_name`) while maintaining IR structure integrity.

### Known Limitations
- Addition operations currently use `max(bits)` instead of `bits + 1` for carry bit handling
- This is a known limitation that may be addressed in future versions
