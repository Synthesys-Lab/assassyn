# Expression IR Nodes

This module provides the foundational Abstract Syntax Tree data structures for expressions in the Assassyn IR. It defines the base `Expr` class and various expression node types that represent different operations in the hardware description language.

The expressions form a use-def graph where each `Expr` tracks its operands through `Operand` wrappers, enabling dataflow analysis and optimization.

---

## Exposed Interfaces

### Core Classes

#### `class Expr(Value)`

The base class for all expression nodes in the IR. It serves as the foundation for operations and builds a use-def graph by tracking its inputs as `Operand`s.

**Fields:**
- `opcode: int` - Operation code for this expression
- `loc: str` - Source location information  
- `parent: typing.Optional[Block]` - Parent block of this expression
- `users: typing.List[Operand]` - List of users of this expression
- `_operands: typing.List[typing.Union[Operand, Port, Array, int]]` - List of operands of this expression

**Methods:**
- `__init__(opcode, operands: list)` - Initialize the expression with an opcode
- `get_operand(idx: int)` - Get the operand at the given index
- `operands` - Get the operands of this expression (property)
- `as_operand()` - Dump the expression as an operand string
- `is_binary()` - Check if the opcode is a binary operator
- `is_unary()` - Check if the opcode is a unary operator  
- `is_valued()` - Check if this operation has a return value

#### `class Operand`

A wrapper that creates a directed link between a value and the `Expr` that consumes it. This is the core mechanism for tracking dataflow dependencies.

**Fields:**
- `_value: Value` - The value of this operand
- `_user: typing.Union[Expr, CondBlock]` - The user of this operand

**Methods:**
- `__init__(value: Value, user: Expr)` - Initialize the operand
- `value` - Get the value of this operand (property)
- `user` - Get the user of this operand (property)
- `__getattr__(name)` - Forward attribute access to the value

### Expression Node Types

#### `class FIFOPop(Expr)`

Represents consuming a value from a port's FIFO. The resulting data type is derived from the port's type.

**Constants:**
- `FIFO_POP = 301`

**Methods:**
- `__init__(fifo)` - Initialize FIFO pop operation
- `fifo` - Get the FIFO port (property)
- `dtype` - Get the data type of the popped value (property)

#### `class Concat(Expr)`

Represents the bit-concatenation of two values. The result's bit width is the sum of the operand widths.

**Constants:**
- `CONCAT = 701`

**Methods:**
- `__init__(msb, lsb)` - Initialize concatenation operation
- `msb` - Get the most significant bit (property)
- `lsb` - Get the least significant bit (property)
- `dtype` - Get the data type of the concatenated value (property)

#### `class Cast(Expr)`

Represents type conversions, including `bitcast`, zero-extend (`zext`), and sign-extend (`sext`) to a specified target type.

**Constants:**
- `BITCAST = 800`
- `ZEXT = 801` 
- `SEXT = 802`

**Fields:**
- `dtype: DType` - Target data type

**Methods:**
- `__init__(subcode, x, dtype)` - Initialize cast operation
- `x` - Get the value to cast (property)

#### `class Select(Expr)`

Represents a ternary multiplexer that chooses between two values of the same type based on a condition.

**Constants:**
- `SELECT = 1000`

**Methods:**
- `__init__(opcode, cond, true_val: Value, false_val: Value)` - Initialize select operation
- `cond` - Get the condition (property)
- `true_value` - Get the true value (property)
- `false_value` - Get the false value (property)
- `dtype` - Get the data type of this operation (property)

#### `class Select1Hot(Expr)`

A specialized multiplexer controlled by a one-hot encoded signal.

**Constants:**
- `SELECT_1HOT = 1001`

**Methods:**
- `__init__(opcode, cond, values)` - Initialize 1hot select operation
- `cond` - Get the condition (property)
- `values` - Get the list of possible values (property)
- `dtype` - Get the data type of this operation (property)

#### `class WireAssign(Expr)`

Represents wire assignment operations for external connections.

**Constants:**
- `WIRE_ASSIGN = 1100`

**Methods:**
- `__init__(wire, value)` - Initialize wire assignment
- `wire` - Get the wire being assigned to (property)
- `value` - Get the value being assigned (property)

#### `class WireRead(Expr)`

Represents reading from an external wire.

**Constants:**
- `WIRE_READ = 1101`

**Methods:**
- `__init__(wire)` - Initialize wire read operation
- `wire` - Return the wire being read (property)
- `dtype` - The data type carried by the wire (property)

#### `class Log(Expr)`

A non-synthesizable node that functions as a print statement for debugging during simulation.

**Constants:**
- `LOG = 600`

**Fields:**
- `args: tuple` - Arguments to the log operation

**Methods:**
- `__init__(*args)` - Initialize log operation

### Frontend Functions

#### `def log(*args) -> Log`

The exposed frontend function to instantiate a log operation.

**Parameters:**
- `*args` - Variable arguments for the log operation

**Returns:**
- `Log` - The log expression node

**Explanation:**
This function creates a `Log` expression node for debugging purposes. The first argument must be a string format, followed by values to be logged. This is non-synthesizable and only works during simulation.

#### `def wire_assign(wire, value) -> WireAssign`

Create a wire assignment expression.

**Parameters:**
- `wire` - The wire to assign to
- `value` - The value to assign

**Returns:**
- `WireAssign` - The wire assignment expression node

#### `def wire_read(wire) -> WireRead`

Create a wire read expression.

**Parameters:**
- `wire` - The wire to read from

**Returns:**
- `WireRead` - The wire read expression node

---

## Internal Helpers

### Location Information

All `Expr` nodes include accurate source location information captured by the `@ir_builder` decorator. The location format is `filename:lineno:col` where:
- `filename`: The Python source file where the expression was created
- `lineno`: The line number in the source file  
- `col`: The accurate column position within the line

This information is displayed in IR dumps as comments in the format `; <filename:lineno:col>` after each expression, providing precise debugging information for developers.

### Use-Def Graph Construction

The `Expr` constructor automatically builds the use-def graph by:
1. Wrapping values in `Operand` objects to track dependencies
2. Adding the current expression to the users list of each operand
3. Handling special cases for different operand types (Array, Port, Wire, Module, Downstream)

### Valued Operations

The `is_valued()` method determines if an expression produces a value that can be used by other expressions. It returns `True` for:
- `PureIntrinsic` operations
- `FIFOPop` operations  
- `ArrayRead` operations
- `Slice` operations
- `Cast` operations
- `Concat` operations
- `Select` operations
- `Select1Hot` operations
- `WireRead` operations
- Binary and unary operations
- `Intrinsic` operations that may return values (like `send_write_request` and `send_read_request`)
