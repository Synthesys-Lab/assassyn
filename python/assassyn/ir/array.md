# Array Module

The `array.py` module defines the `Array` class for representing register arrays in the Assassyn IR, along with the `RegArray` function for declaring them. Register arrays are fundamental data structures that store multiple values of the same type, accessible via indexing operations.

## Summary

This module provides the core infrastructure for register arrays in Assassyn's IR. Register arrays are used extensively throughout the system for storing stateful data, implementing memory structures, and managing pipeline stage registers. The module supports both single-port and multi-port access patterns through the `WritePort` mechanism, enabling multiple modules to write to the same array while maintaining proper hardware semantics.

Register arrays in Assassyn follow a multi-port access model where each module that writes to an array gets its own dedicated write port. This is implemented through the syntactic sugar `(array & module)[index] <= value`, which creates a `WritePort` object that handles the write operation. The system automatically manages port conflicts and ensures proper hardware semantics during code generation.

The module also provides the `Slice` class for bit-slicing operations, which is essential for extracting specific bit fields from wider values in hardware design.

**Type System Integration:** Each `Array` instance has a `dtype` property that returns an `ArrayType` instance representing the array's type in the type system. This connects the runtime register array representation with the static type system defined in [dtype.md](dtype.md#arraytypedtype-size---array-type).

## Exposed Interfaces

The `array.py` module provides the `RegArray` function and `Array` class methods for creating and manipulating register arrays.

### Ownership Metadata

Each array records its provenance via the `owner` attribute. The owner is one of:

- `None` when the array is instantiated outside any module.
- A `ModuleBase` instance for module-scoped arrays.
- A `MemoryBase` instance (for example, `SRAM` or `DRAM`) for memory-managed
  buffers.

Downstream passes should use `Array.is_payload(memory_cls_or_instance)` to detect
memory payload buffers. The helper consolidates the identity logic that used to
be implemented ad hoc (`array.owner is memory and array is memory._payload`),
while auxiliary registers such as the SRAM `dout` latch continue to look like
regular arrays even though they reference the same owner. See
[`docs/design/internal/array-ownership.md`](../../../docs/design/internal/array-ownership.md)
for the detailed rationale.

### `RegArray`

```python
def RegArray(
    scalar_ty: DType,
    size: int,
    initializer: list = None,
    name: str = None,
    attr: list = None,
    *,
    owner: ModuleBase | MemoryBase | None = None,
) -> Array:
    '''
    The frontend API to declare a register array.

    @param scalar_ty The data type of the array elements.
    @param size The size of the array. MUST be a compilation time constant.
    @param initializer The initializer of the register array. If not set, it is 0-initialized.
    @param name The custom name for the array.
    @param attr The attribute list of the array.
    @param owner Optional ownership override; defaults to the current module (or None outside a module).
    @return Array instance registered with the AST builder.
    '''
```

**Explanation:**

This function serves as the primary interface for creating register arrays in Assassyn. It creates an `Array` instance and automatically registers it with the global builder singleton for proper IR construction. The function handles naming semantics by integrating with the [naming manager](../builder/naming_manager.md) to provide meaningful names when no explicit name is given.

The naming behavior follows a hierarchical approach:
- If `name` is provided, it is sanitized using [namify](../../utils.md#namify) and applied directly
- If no explicit name is given and a module context is active, a semantic name is assigned using the module name as a prefix (e.g., `<module>_array`)
- Semantic names are stored on the instance and used by `as_operand()` and `__repr__` methods

The function automatically adds the created array to the builder's `arrays` list, which is used during code generation to emit array declarations and manage write ports. The `attr` parameter allows attaching metadata to arrays, which is commonly used to associate arrays with their parent modules (e.g., in memory modules).

**Examples:**
```python
# Basic array declaration
my_array = RegArray(UInt(32), 16, name="register_file")  # 16-element array of 32-bit uints

# Array with initial values
counter = RegArray(Int(32), 1, initializer=[0])  # Single-element counter initialized to 0

# Array with attributes (commonly used in memory modules)
payload = RegArray(
    Bits(64),
    1024,
    attr=[self],
    name=f'{self.name}_val',
    owner=self,  # assign memory instance as owner
)
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
    _owner: 'ModuleBase | MemoryBase | None'  # Provenance descriptor
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

The name property implements a simplified naming system that uses the internal `_name` field and generates a default name using [identifierize](../../utils.md#identifierize) if no name is set.

This naming system is crucial for code generation, as the array name is used in both Verilog and Rust simulator output. The unified naming system allows the generated code to have meaningful, hierarchical names that reflect the module structure, making debugging and analysis easier.

#### `dtype` Property

```python
@property
def dtype(self) -> ArrayType:
    '''
    Get the data type of the array as an ArrayType.

    @return ArrayType instance representing the array's type.
    '''
```

**Explanation:**

This property provides the connection between the runtime `Array` instance and the type system's `ArrayType`. It returns an `ArrayType(self.scalar_ty, self.size)` that represents the array's type information in the static type system. This is useful for type checking, code generation, and ensuring consistency between runtime and compile-time representations.

The `ArrayType` provides the same `scalar_ty` and `size` information as the `Array` instance but in a form suitable for type system operations. See [ArrayType documentation](dtype.md#arraytypedtype-size---array-type) for more details on the type system representation.

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

The method maintains a dictionary `_write_ports` that maps each module to its dedicated `WritePort` instance. This ensures that each module gets its own write port, enabling proper multi-port access during code generation. The Verilog code generator uses this information to create separate write enable, write data, and write index signals for each port.

The method also supports fallback to regular bitwise AND operations when the operand is a `Value`, maintaining compatibility with standard Python operations. This dual behavior allows the same operator to serve both hardware-specific multi-port access and general bitwise operations.

**Usage Pattern:**
```python
# Multi-port write access
(array & current_module)[index] <= value

# Regular bitwise AND (fallback)
result = array & some_value
```

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

The calculation uses the bit manipulation `self.size & (self.size - 1) == 0` to detect power-of-2 sizes. For power-of-2 arrays, the index width is `size.bit_length() - 1` (e.g., a 16-element array needs 4 bits, not 5). For non-power-of-2 arrays, it uses `size.bit_length()` to get the minimum bits needed.

This property is crucial for code generation, as it determines the width of address signals in both Verilog and Rust simulator output. The Verilog code generator uses this to create properly sized address ports and internal signals.

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

This method implements array read operations with predicate-aware caching to avoid duplicate reads within active predicate scopes. The caching mechanism is integrated with the predicate frame stack to ensure correct invalidation when predicates are popped.

The method probes predicate frame caches from top (most nested) to bottom, allowing outer-scope reads to be reused within inner predicates while preventing inner-scope reads from leaking out after their predicate expires. If a cached read is found in any active predicate frame, it is returned immediately. Otherwise, a new `ArrayRead` is created and stored in the top-most active predicate frame's cache.

The method automatically converts integer indices to `UInt` values using [to_uint](../dtype.md#to_uint) and creates `ArrayRead` expressions that represent the read operation in the IR.

The cache key is a tuple of `(array, index)`, ensuring that different indices into the same array are treated as separate operations, while the same index access within a predicate scope is deduplicated. This predicate-aware caching is essential for FSM and other conditional execution patterns where array reads must not leak across conditional boundaries.

**Cache Protocol:**
- Cache probing: Walks the predicate frame stack from top to bottom
- Cache insertion: New reads are stored in the top-most active predicate frame
- Cache invalidation: Automatically handled when predicates are popped
- Cache scope: Per-predicate frame, ensuring no leakage across conditional boundaries

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
    Enforces strict type checking between the written value and array's scalar_ty.

    @param index Integer or Value for the array index.
    @param value Value or RecordValue to write.
    @raises TypeError If value type doesn't match array's scalar_ty.
    '''
```

**Explanation:**

This method implements array write operations by creating a write port for the current module and delegating to the write port's `_create_write` method. It automatically handles the conversion of integer indices to `Value` objects and ensures proper type checking for the value being written.

**Type Checking:** The method enforces strict type equality between the written value and the array's element type (`scalar_ty`). This includes:

- **Regular Values**: The value's `dtype` must exactly match the array's `scalar_ty` using `type_eq()` method
- **RecordValue Handling**: RecordValue objects are automatically unwrapped to their underlying `Bits` representation before type checking and write creation
- **Error Messages**: Type mismatches raise `TypeError` with detailed information about expected vs actual types

The write operation uses the `&` operator internally to create or retrieve the appropriate `WritePort` for the current module context. This ensures that each module gets its own dedicated write port, enabling proper multi-port access semantics.

**Usage Pattern:**
```python
# Direct write using current module's write port
array[index] = value

# This is equivalent to:
# (array & current_module)[index] <= value
```

**Type Checking Examples:**
```python
# Correct: matching types
array = RegArray(UInt(8), 4)
value = Const(UInt(8), 42)
array[0] = value  # ✓ Succeeds

# Error: type mismatch
wrong_value = Const(UInt(16), 42)
array[0] = wrong_value  # ✗ Raises TypeError

# RecordValue: automatically unwrapped
rec_type = Record(a=UInt(8), b=UInt(16))
rec_array = RegArray(rec_type, 4)
rec_value = RecordValue(rec_type, a=Const(UInt(8), 1), b=Const(UInt(16), 2))
rec_array[0] = rec_value  # ✓ Succeeds, unwraps to Bits
```

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

The slice operation is fundamental in hardware design for:
- Extracting control bits from wider values (e.g., `value[0:0]` for a single bit)
- Accessing specific fields in packed data structures
- Implementing bit-level operations and manipulations

The class inherits from `Expr`, making it a first-class expression in the IR that can be used in arithmetic operations, assignments, and other expressions.

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

The calculation assumes that both `l` and `r` are `Const` values (compile-time constants), which is enforced by the constructor. The resulting `Bits` type represents an unsigned bit vector of the extracted width, which is the standard representation for bit-sliced values in hardware.

This property is used during IR construction to ensure type consistency and during code generation to determine the correct signal widths in the generated Verilog and Rust code.

#### `__repr__`

```python
def __repr__(self):
    '''
    String representation of the slice operation.

    @return Formatted string showing the slice operation.
    '''
```
#### `owner` Property and `assign_owner`

```python
@property
def owner(self) -> ModuleBase | MemoryBase | None:
    '''
    Return the ownership context for this array.

    @return ModuleBase, MemoryBase, or None describing provenance.
    '''

def assign_owner(self, owner: ModuleBase | MemoryBase | None) -> None:
    '''
    Override the ownership context.

    @param owner Ownership reference to apply. Intended for controlled refactors.
    '''

def is_payload(self, memory: type['MemoryBase'] | 'MemoryBase') -> bool:
    '''
    Return whether this array is the payload buffer of the provided memory.

    @param memory Memory class (SRAM/DRAM) or instance to test against.
    @return True when self.owner matches the memory type and the array is the memory payload.
    '''
```

**Explanation:**

`owner` exposes the provenance metadata described in the ownership model
documentation, and `assign_owner` provides the sanctioned hook for controlled
refactors that need to re-home an array while keeping type validation in place.
The `is_payload` helper is the canonical way to detect memory payload buffers:
it accepts either a memory class (`SRAM`, `DRAM`) or an instance of those
classes. Internally the argument is normalised into a `(memory_cls,
memory_instance)` pair so that type validation, owner comparison, and payload
identity checks flow through the same code path. Supplying any other type
raises the same `TypeError` regardless of whether the caller passed a class or
an instance, keeping misuse obvious while avoiding duplicated logic.

**Example:**

```python
sram = SRAM(width=64, depth=1024, init_file=None)
assert sram._payload.is_payload(SRAM)
assert sram._payload.is_payload(sram)
assert not sram.dout.is_payload(SRAM)
```
