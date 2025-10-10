# Array Module
The `array.py` module defines the `Array` class for representing register arrays in the Assassyn IR, along with the `RegArray` function for declaring them.

```python
class Array:
    scalar_ty: DType  # Data type of each element in the array
    size: int         # Size of the array
    initializer: list # Initial values for the array elements
    attr: list       # Attributes of the array
    _users: typing.List[Expr]  # Users of the array
    _name: str       # Internal name storage
    _write_ports: typing.Dict['ModuleBase', 'WritePort']  # Write ports for this array
```

## Exposed Interface
The `array.py` module provides the `RegArray` function and `Array` class methods for creating and manipulating register arrays.

### `RegArray(scalar_ty, size, initializer=None, name=None, attr=None)`
```python
def RegArray(  # Frontend API to declare a register array
    scalar_ty: DType,  # Data type of array elements
    size: int,         # Array size, must be compile-time constant
    initializer: list = None,  # Initial values, defaults to 0-initialized
    name: str = None,  # Custom name for the array
    attr: list = None  # Attribute list for the array
) -> Array
```
- **Description:** Creates a register array and registers it with the AST builder.
- **Returns:** `Array` instance.
- **Example:**
  ```python
  my_array = RegArray(UInt(32), 16, name="register_file")  # 16-element array of 32-bit uints
  ```

#### Naming behavior
- If `name` is provided, it is sanitized and applied.
- If no explicit name is given and a module context is active, a semantic name is assigned using the module name as a prefix (e.g. `<module>_array`).
- Semantic names are stored on the instance and used by `as_operand()` and `__repr__`.

-------

### `as_operand(self)`
```python
def as_operand(self) -> str  # Dump the array as an operand
```
- **Description:** Returns the array’s name for use in IR expressions.
- **Returns:** String representing the array’s name.

-------

### `name` Property
```python
@property
def name(self) -> str  # Get array name, auto-generated if not set
@name.setter
def name(self, name: str)  # Set custom array name
```
- **Description:** Gets or sets the array’s name, defaulting to `array_{unique_identifier}`.
- **Returns:** String name of the array.

-------

### `users` Property
```python
@property
def users(self) -> typing.List[Expr]  # Get expressions using the array
```
- **Description:** Returns the list of expressions that reference the array.
- **Returns:** List of `Expr` objects.

-------

### `__and__(self, other)`
```python
def __and__(self, other) -> WritePort | BinaryOp  # Create write port or perform bitwise AND
```
- **Description:** Creates a `WritePort` when combined with a `ModuleBase` for write access, or performs bitwise AND with a `Value`.
- **Parameters:**
  - `other`: A `ModuleBase` or `Value`.
- **Returns:** `WritePort` or `BinaryOp`.
- **Example:**
  ```python
  write_port = array & module  # Create write port for module
  ```

-------

### `__repr__(self)`
```python
def __repr__(self) -> str  # String representation with write port info
```
- **Description:** Returns a string representation of the array, including name, type, size, and write ports.
- **Returns:** Formatted string, e.g., `array register_file[UInt(32); 16]`.



### `index_bits` Property
```python
@property
def index_bits(self) -> int  # Bits needed to index the array
```
- **Description:** Calculates the number of bits required for indexing, optimized for power-of-2 sizes.
- **Returns:** Integer bit count.

-------

### `index_type(self)`
```python
def index_type(self) -> UInt  # Get index type
```
- **Description:** Returns the `UInt` type for array indexing based on `index_bits`.
- **Returns:** `UInt` type.

-------

### `get_write_ports(self)`
```python
def get_write_ports(self) -> typing.Dict['ModuleBase', 'WritePort']  # Get write ports
```
- **Description:** Returns the dictionary of module-specific write ports.
- **Returns:** Dictionary mapping modules to `WritePort` objects.

-------

### `__getitem__(self, index)`
```python
@ir_builder
def __getitem__(self, index: typing.Union[int, Value]) -> ArrayRead  # Read from array
```
- **Description:** Reads from the array at the specified index, creating an `ArrayRead` expression.
- **Parameters:**
  - `index`: Integer or `Value` for the array index.
- **Returns:** `ArrayRead` expression.
- **Example:**
  ```python
  value = array[address]  # Read value at address
  ```

-------

### `get_flattened_size(self)`
```python
def get_flattened_size(self) -> int  # Total bits in array
```
- **Description:** Calculates the total bit count of the array (`size * scalar_ty.bits`).
- **Returns:** Integer total bits.

-------

### `__setitem__(self, index, value)`
```python
@ir_builder
def __setitem__(self, index, value)  # Write to array
```
- **Description:** Writes a value to the array at the specified index using the current module’s write port.
- **Parameters:**
  - `index`: Integer or `Value` for the array index.
  - `value`: `Value` or `RecordValue` to write.
- **Example:**
  ```python
  array[address] = new_value  # Write value to address
  ```