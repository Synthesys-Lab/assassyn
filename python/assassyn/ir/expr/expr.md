# Expression IR Nodes

This module provides the foundational Abstract Syntax Tree data structures for expressions in the Assassyn IR. It defines the base `Expr` class and various expression node types that represent different operations in the hardware description language.

The expressions form a use-def graph where each `Expr` tracks its operands through `Operand` wrappers, enabling dataflow analysis and optimization.

## Design Philosophy

Assassyn's IR employs an intentionally **highly redundant** data structure design that pays the cost of redundancy during construction to enable fast, simple queries during all subsequent compiler passes. This design philosophy centers around **bidirectional use-def relationships** established through operand wrappers.

### Bidirectional Use-Def Graph

Every value in the IR maintains bidirectional relationships:
- **Forward edge**: `Expr._operands` contains `Operand` wrappers pointing to the values used
- **Backward edge**: Each value's `users` list accumulates the `Operand` wrappers that reference it

This means **every value knows all of its consumers immediately after IR construction**, without requiring separate analysis passes.

### Design Trade-off

**Cost**: Additional memory for wrapper objects and users lists
**Benefit**: Pay once during construction, then get O(1) queries forever

This pattern is similar to SSA form in traditional compilers but applied more comprehensively across all IR nodes. The redundancy enables critical compiler operations like dead code elimination, dataflow analysis, and value lifetime analysis with simple, fast lookups.

---

## Exposed Interfaces

### Core Classes

#### `class Expr(Value)`

The base class for all expression nodes in the IR. It serves as the foundation for operations and builds a use-def graph by tracking its inputs as `Operand`s.

**Fields:**
- `opcode: int` - Operation code for this expression
- `loc: str` - Source location information  
- `parent: typing.Optional[ModuleBase]` - Owning module of this expression (set by the builder)
- `users: typing.List[Operand]` - List of users of this expression
- `_operands: typing.List[typing.Union[Operand, Port, Array, int]]` - List of operands of this expression
- `_meta_cond: Value | None` - The cumulative predicate (`AND` of active predicate conditions) captured when the node is created. When the builder has an active predicate stack, this references the top frame’s `carry`. If no builder is active, the field remains `None`. Callers may override it explicitly via the `meta_cond` keyword argument.

**Methods:**
- `__init__(opcode, operands: list, *, meta_cond: Value | None = None)` - Initialize the expression with an opcode. Unless an explicit `meta_cond` is supplied, the constructor queries the builder’s predicate stack (if available) and stores the top frame’s `carry`. When the stack is empty, the implicit metadata defaults to the constant predicate `Bits(1)(1)` so downstream consumers can treat `meta_cond` uniformly.
- `get_operand(idx: int)` - Get the operand at the given index
- `operands` - Get the operands of this expression (property)
- `as_operand()` - Dump the expression as an operand string
- `is_binary()` - Check if the opcode is a binary operator
- `is_unary()` - Check if the opcode is a unary operator  
- `is_valued()` - Check if this operation has a return value
- `meta_cond` - Return the stored predicate value guarding this expression. Always resolves to a `Bits(1)` constant `1` when no guard was present at construction time (property)

Internally, the constructor normalizes operands through `_prepare_operand`. Direct references to `Array` or `Port` objects are registered with the operand's `users` list. Expression operands must originate from the same module unless `_is_cross_module_allowed()` explicitly approves the reference. Today the only cross-module exceptions are `PureIntrinsic` nodes for external output reads and `ExternalIntrinsic` handles, which let external SystemVerilog modules share outputs without relaxing other invariants.

#### `class Operand`

A wrapper that creates a **bidirectional link** between a value and the `Expr` that consumes it. This is the core mechanism for tracking dataflow dependencies and enabling the highly redundant use-def graph.

**Bidirectional Relationship:**
- **Forward edge**: The `Operand` wrapper in `Expr._operands` points to the value being used
- **Backward edge**: The value's `users` list contains this `Operand` wrapper, establishing the inverse relationship

**Why Wrap Instead of Direct Storage:**
Direct storage would only provide forward traversal (Expr → Value). The wrapper enables both directions:
- Need to track the value AND establish the inverse relationship
- The wrapper is what enables "all values know all their consumers immediately after construction"
- This pattern is used consistently across expressions, blocks, arrays, ports, and wires

**Fields:**
- `_value: Value` - The value of this operand
- `_user: Expr` - The expression that consumes this operand

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
- `__init__(fifo, meta_cond=None)` - Initialize FIFO pop operation
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

#### `class Log(Expr)`

A non-synthesizable node that functions as a print statement for debugging during simulation.

**Constants:**
- `LOG = 600`

**Fields:**
- `args: tuple` - Positional arguments backing the operation.

**Properties:**
- `fmt` - Returns the format string (`args[0]`).
- `values` - Returns the payload values to be substituted into the format string (`args[1:-1]`).
- `dtype` - Get the data type of this operation (void for side-effect operations).

**Methods:**
- `__init__(fmt, *values, meta_cond)` - Initialize log operation

**Notes:**
- `Log` is an ordinary expression node, **not** an intrinsic. The frontend helper relies on the base `Expr` constructor to capture the current predicate carry in `meta_cond` so downstream tools can gate traces without reconstructing the predicate stack.

### Frontend Functions

#### `def log(*args) -> Log`

The exposed frontend function to instantiate a log operation.

**Parameters:**
- `*args` - Variable arguments for the log operation

**Returns:**
- `Log` - The log expression node

**Explanation:**
This function creates a `Log` expression node for debugging purposes. The first argument must be a string format, followed by values to be logged. It is non-synthesizable and only works during simulation.

On creation the helper captures the builder’s current predicate carry and stores it in `meta_cond`, letting backends reuse the same guard without threading extra operands or reconstructing the predicate stack.


---

## Internal Helpers

### Location Information

All `Expr` nodes include accurate source location information captured by the `@ir_builder` decorator. The location format is `filename:lineno:col` where:
- `filename`: The Python source file where the expression was created
- `lineno`: The line number in the source file  
- `col`: The accurate column position within the line

This information is displayed in IR dumps as comments in the format `; <filename:lineno:col>` after each expression, providing precise debugging information for developers.

### Use-Def Graph Construction

The `Expr` constructor automatically builds the bidirectional use-def graph through the `_prepare_operand` method:

**For Expression Operands:**
```python
if isinstance(operand, Expr):
    wrapped = Operand(operand, self)  # Create wrapper
    operand.users.append(wrapped)     # Establish backward edge
    return wrapped                    # Return wrapper for forward edge
```

**For Array/Port Operands:**
```python
if isinstance(operand, (Array, Port)):
    operand.users.append(self)        # Direct backward edge
    return operand                     # No wrapper needed
```

**Key Points:**
- Expression operands are wrapped in `Operand` objects to establish bidirectional links
- Array and Port operands are stored directly but still register backward edges
- The `operand.users.append(wrapped)` call is what establishes the backward edge
- Different operand types have slightly different handling based on their nature
- This automatic construction happens during IR creation, not in separate analysis passes

### Benefits of Bidirectional Use-Def Graph

The intentional redundancy in the use-def graph enables critical compiler operations:

**Dataflow Analysis:**
- Forward traversal: Follow `_operands` to see what values an expression uses
- Backward traversal: Follow `users` to see what expressions use a value
- Both directions available with O(1) lookups

**Dead Code Elimination:**
- Check `len(value.users) == 0` to identify unused values
- No separate analysis pass needed—the information is already there
- Immediate identification of dead code during IR construction

**Value Lifetime Analysis:**
- Each value knows exactly where it's consumed
- Critical for register allocation and scheduling in hardware synthesis
- Enables precise timing analysis for pipeline stages

**Dependency Tracking:**
- Essential for predicate-managed control flow because predicate intrinsics still consume the condition operands
- Each predicate push intrinsic wraps its condition in an `Operand` to track the dependency
- Enables proper ordering of operations across different execution contexts

**Multi-Port Write Tracking:**
- Arrays track all their users to manage multiple write ports
- Each module gets its own `WritePort` mapped through the `_write_ports` dictionary
- Enables proper hardware semantics for concurrent access patterns

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
- Binary and unary operations
- `Intrinsic` operations that may return values (like `send_write_request` and `send_read_request`)
