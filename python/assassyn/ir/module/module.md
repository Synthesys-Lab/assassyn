# Module and Port Definitions

## Summary

This module provides the core Abstract Syntax Tree (AST) implementations for hardware modules and ports in Assassyn's credit-based pipeline architecture. The `Module` class represents the primary pipeline stage (note: this is a legacy naming issue - modules are actually pipeline stages in the current architecture), while the `Port` class handles communication interfaces as described in the [module design](../../../docs/design/internal/module.md) and [pipeline design](../../../docs/design/internal/pipeline.md).

## Exposed Interfaces

### Module Class

```python
class Module(ModuleBase):
    def __init__(self, ports, no_arbiter=False): ...
    @property
    def users(self): ...
    @property
    def ports(self): ...
    def validate_all_ports(self): ...
    def pop_all_ports(self, validate): ...
    @ir_builder
    def async_called(self, **kwargs): ...
    @ir_builder
    def bind(self, **kwargs): ...
    def __repr__(self): ...
    @property
    def is_systolic(self): ...
    @property
    def timing(self): ...
    @timing.setter
    def timing(self, value): ...
    @property
    def no_arbiter(self): ...
```

### Port Class

```python
class Port:
    def __init__(self, dtype: DType): ...
    def __class_getitem__(cls, item): ...
    @property
    def users(self): ...
    @ir_builder
    def valid(self): ...
    @ir_builder
    def peek(self): ...
    @ir_builder
    def pop(self): ...
    @ir_builder
    def push(self, v): ...
    def __repr__(self): ...
    def as_operand(self): ...
```

### Combinational Decorator

```python
@combinational
def my_module_logic(self, ...):
    """
    Define the module's logic using combinational operations.
    
    @param self The module instance
    @param ... Additional parameters as needed
    """
    ...
```

## Internal Helpers

### Module Class

The `Module` class is the primary AST node for defining hardware modules (pipeline stages) in Assassyn's architecture.

**Purpose:** Represents a pipeline stage that can be activated through credit-based flow control, handles port-based communication, and provides timing policy management for different execution models.

**Member Fields:**
- `body: list[Expr]` - Ordered list of expressions that make up the module's logic
- `name: str` - The module's name
- `_attrs: dict` - Dictionary of module attributes (timing, arbiter settings, etc.)
- `_ports: list` - List of port objects
- `_users: typing.List[Expr]` - List of expressions that use this module

**Methods:**

#### `__init__(self, ports, no_arbiter=False)`

**Explanation:**
Constructs a module with the specified ports and attributes. The constructor:

1. **Base Initialization:** Calls `ModuleBase.__init__()` to initialize external dependency tracking
2. **Name Assignment:** Uses the naming manager if available, or generates a default name based on the class name
3. **Reserved Name Handling:** Special handling for reserved names like 'Driver' and 'Testbench'
4. **Attribute Setup:** Initializes the attributes dictionary and sets the no_arbiter flag if specified
5. **Port Registration:** Creates port objects and registers them as module attributes
6. **System Registration:** Registers the module with the system builder for code generation

The method ensures proper integration with Assassyn's naming system and builder infrastructure.

#### `validate_all_ports(self)`

**Explanation:**
Syntactic sugar for checking if all port FIFOs contain valid data. The method:
1. Iterates through all ports and checks their validity using `port.valid()`
2. Combines all validity signals using bitwise AND operations
3. Calls `wait_until(valid)` to suspend execution until all ports are valid

This method is commonly used in backpressure timing modules to ensure all inputs are available before processing.

#### `pop_all_ports(self, validate)`

**Explanation:**
Syntactic sugar for consuming all port data at once. The method:
1. **Timing Policy Setting:** Sets the module's timing policy based on the validate parameter
2. **Validation:** Optionally calls `validate_all_ports()` if validation is requested
3. **Data Consumption:** Pops data from all ports and returns the results

Returns a single value if only one port exists, or a list of values for multiple ports.

#### `async_called(self, **kwargs)`

**Explanation:**
Frontend API for creating an async call operation to this module. The method:
1. Creates a bind operation using `self.bind(**kwargs)`
2. Wraps the bind in an `AsyncCall` expression
3. Returns the async call expression for use in the IR

This method enables credit-based activation of the module from other pipeline stages.

#### `bind(self, **kwargs)`

**Explanation:**
Frontend API for creating a bind operation to this module. The method:
1. Creates a `Bind` expression with the module and provided keyword arguments
2. Returns the bind expression for use in async calls or other operations

The bind operation establishes the connection between the caller and this module's ports.

#### `__repr__(self)`

**Explanation:**
Generates the string representation for IR dumps. The method:
1. Formats port definitions with proper indentation
2. Includes module attributes in the header
3. Renders the module body via the shared `render_module_body()` helper, which iterates the flat expression list and interprets predicate push/pop intrinsics as braces for display purposes
4. Includes external dependencies

The output follows Assassyn's IR format for module declarations while reflecting the predicate-based structure encoded in the expression stream.

#### `timing` property

**Explanation:**
Provides access to the module's timing policy. The getter returns the current timing policy from attributes, while the setter:
1. Validates that timing is not set twice
2. Converts string values to enum values if needed
3. Stores the timing policy in the attributes dictionary

Timing policies control how the module handles port data consumption and execution flow.

### Port Class

The `Port` class defines typed communication interfaces for modules.

**Purpose:** Provides FIFO-based communication between modules, supporting both systolic and backpressure timing models.

**Member Fields:**
- `dtype: DType` - The data type of the port
- `name: str` - The port's name
- `module: Module` - The module this port belongs to
- `_users: typing.List[Expr]` - List of expressions that use this port

**Methods:**

#### `__init__(self, dtype: DType)`

**Explanation:**
Initializes a port with the specified data type. The constructor:
1. Validates that the dtype is a proper `DType` object
2. Initializes name and module references to None
3. Creates an empty users list

#### `__class_getitem__(cls, item)`

**Explanation:**
Enables subscriptable syntax like `Port[UInt(32)]` for type annotations. Returns a new `Port` instance with the specified dtype.

#### `valid(self)`

**Explanation:**
Frontend API for checking if the port's FIFO contains valid data. Returns a `PureIntrinsic` expression that checks FIFO validity.

#### `peek(self)`

**Explanation:**
Frontend API for reading data from the port's FIFO without consuming it. Returns a `PureIntrinsic` expression for FIFO peek operations.

#### `pop(self)`

**Explanation:**
Frontend API for consuming data from the port's FIFO. Returns a `FIFOPop` expression that removes and returns the next value.

#### `push(self, v)`

**Explanation:**
Frontend API for pushing data into the port's FIFO. Returns a `FIFOPush` expression that adds the value to the FIFO.

### Combinational Decorator

The `@combinational` decorator is created by `combinational_for(Module)` from [base.py](base.md).

**Explanation:**
This decorator provides essential functionality for module logic definition:

1. **IR Context Management:** Automatically enters and exits module and body contexts
2. **Signal Naming:** Infers signal names from Python source code parameters
3. **AST Transformation:** Uses `rewrite_assign` to handle assignment operations
4. **Body Assignment:** Creates and assigns the list object that stores the module's logic

The decorator ensures that all operations within the decorated function are properly recorded in the IR and that the module's body is correctly structured for code generation.
