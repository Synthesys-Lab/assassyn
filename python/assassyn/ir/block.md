# Block Module

## Section 0. Summary

The `block.py` module defines the `Block` class hierarchy for representing control flow blocks in the Assassyn IR. This module implements the block-based control flow system that works with the [builder singleton](../../builder/__init__.py) to manage the current insertion point for IR nodes. Blocks serve as containers for expressions and provide context management through Python's `with` statement, enabling conditional execution and testbench cycle-based execution patterns as described in the [DSL design](../../../docs/design/dsl.md).

## Section 1. Exposed Interfaces

This section describes all the function interfaces and data structures that are exposed to other parts of the project.

### Data Structures

#### `Block`
```python
class Block:
    kind: int                                         # Kind of block (MODULE_ROOT, CONDITIONAL, CYCLE)
    _body: list[Expr]                                # List of instructions in the block
    parent: typing.Union[typing.Self, ModuleBase]    # Parent block
    module: typing.Optional[ModuleBase]              # Module of this block
```

**Purpose:** Base class for all control flow blocks in the Assassyn IR.

**Member Fields:**
- `kind`: Integer constant defining the block type (MODULE_ROOT, CONDITIONAL, or CYCLE)
- `_body`: List of `Expr` objects representing the instructions contained in this block
- `parent`: Reference to the parent block or module that contains this block
- `module`: Reference to the module that owns this block

**Static Member Fields:**
- `MODULE_ROOT = 0`: Constant for root blocks of modules
- `CONDITIONAL = 1`: Constant for conditional execution blocks  
- `CYCLE = 2`: Constant for cycle-based blocks used in testbench generation

#### `CondBlock`
```python
class CondBlock(Block):
    cond: Value  # Condition for this block
```

**Purpose:** Represents conditional blocks that execute when a condition is true.

**Member Fields:**
- `cond`: `Value` object representing the condition that determines when this block executes

#### `CycledBlock`
```python
class CycledBlock(Block):
    cycle: int  # Cycle count for this block
```

**Purpose:** Represents blocks that execute at specific cycles during testbench generation.

**Member Fields:**
- `cycle`: Integer specifying the cycle number when this block should execute

### Functions

#### `Condition(cond)`
```python
@ir_builder(node_type='expr')
def Condition(cond: Value) -> CondBlock
```

**Description:** Frontend API for creating a conditional block that executes when the condition is true.

**Parameters:**
- `cond`: A `Value` representing the condition to evaluate

**Returns:** `CondBlock` instance that can be used as a context manager

**Explanation:** This function creates a conditional block that integrates with the [builder singleton](../../builder/__init__.py) context management system. When used with a `with` statement, it changes the current insertion point to the conditional block, allowing expressions to be conditionally executed. The condition is evaluated at runtime in the generated hardware, creating a multiplexer-based conditional execution path as described in the [module generation design](../../../docs/design/module.md).

**Example:**
```python
with Condition(enable_signal):
    # Instructions execute when enable_signal is true
    output.next = input_data
```

#### `Cycle(cycle)`
```python
@ir_builder(node_type='expr')
def Cycle(cycle: int) -> CycledBlock
```

**Description:** Frontend API for creating a cycled block for testbench generation that executes at a specific cycle.

**Parameters:**
- `cycle`: Integer cycle number when the block should execute

**Returns:** `CycledBlock` instance that can be used as a context manager

**Explanation:** This function creates a cycle-based block used specifically for testbench generation. The block executes at the specified cycle during simulation, allowing testbench logic to be scheduled at precise timing points. This is used in conjunction with the [simulator design](../../../docs/design/simulator.md) to coordinate testbench events.

**Example:**
```python
with Cycle(10):
    # Instructions execute at cycle 10
    test_signal.next = UInt(1, 1)
```

## Section 2. Internal Helpers

This section describes all the function interfaces and data structures that are implemented internally within this source code unit.

### `Block` Class Methods

#### `__init__(self, kind)`
```python
def __init__(self, kind: int)
```

**Description:** Creates a new block of the specified kind with empty body.

**Parameters:**
- `kind`: Integer constant defining the block type (MODULE_ROOT, CONDITIONAL, or CYCLE)

**Explanation:** Initializes a new block with the specified kind, creates an empty body list, and sets parent and module references to None. This is the base constructor for all block types.

#### `body` Property
```python
@property
def body(self) -> list[Expr]
```

**Description:** Returns the list of instructions contained in the block.

**Returns:** List of `Expr` objects representing the block's instructions.

**Explanation:** Provides read-only access to the block's body. The body is stored as `_body` internally to prevent direct modification, ensuring proper encapsulation.

#### `as_operand(self)`
```python
def as_operand(self) -> str
```

**Description:** Returns a string representation of the block for use as an operand in code generation.

**Returns:** String in the format `_{namified_identifier}`.

**Explanation:** Converts the block to a string representation suitable for use as an operand in generated code. Uses the `namify` and `identifierize` utilities to create a valid identifier.

#### `insert(self, x, elem)`
```python
def insert(self, x: int, elem: Expr)
```

**Description:** Inserts an instruction at the given position in the block's body.

**Parameters:**
- `x`: Integer position where to insert the instruction
- `elem`: `Expr` object to insert

**Explanation:** Directly modifies the block's body by inserting an expression at the specified position. This is used internally by the builder system when managing instruction ordering.

#### `iter(self)`
```python
def iter(self)
```

**Description:** Generator that yields each instruction in the block's body.

**Yields:** `Expr` objects from the block's body.

**Explanation:** Provides iteration support for blocks, allowing them to be used in for loops and other iteration contexts. This is used by the `__repr__` method to display block contents.

#### `__enter__(self)`
```python
def __enter__(self) -> Block
```

**Description:** Sets up the block context when entering a `with` statement.

**Returns:** The block instance for use in the `with` statement.

**Explanation:** Implements the context manager protocol for blocks. Establishes parent-child relationships by assigning the current block/module as parent, sets the module reference from the builder singleton, and switches the builder context to this block. This enables the block to become the current insertion point for new IR nodes. The method includes assertions to ensure safe nesting of blocks.

#### `__exit__(self, exc_type, exc_value, traceback)`
```python
def __exit__(self, exc_type, exc_value, traceback)
```

**Description:** Cleans up the block context when exiting a `with` statement.

**Parameters:** Standard exception handling parameters (exc_type, exc_value, traceback)

**Explanation:** Implements the context manager protocol for blocks. Restores the previous builder context by calling `exit_context_of('block')` on the builder singleton, effectively popping this block from the context stack and returning to the previous insertion point.

#### `__repr__(self)`
```python
def __repr__(self) -> str
```

**Description:** Returns a formatted string representation of the block with proper indentation for nested structures.

**Returns:** Indented string showing the block's contents.

**Explanation:** Creates a formatted string representation of the block for debugging and display purposes. Uses the `Singleton.repr_ident` to maintain proper indentation levels for nested blocks. The representation shows all expressions contained in the block's body.

### `CondBlock` Class Methods

#### `__init__(self, cond)`
```python
def __init__(self, cond: Value)
```

**Description:** Creates a conditional block that executes when the condition is true.

**Parameters:**
- `cond`: `Value` representing the condition to evaluate

**Explanation:** Initializes a conditional block by calling the parent constructor with `Block.CONDITIONAL` kind. Wraps the condition in an `Operand` object and establishes the user relationship if the condition is an expression. This ensures proper dependency tracking in the IR.

#### `__repr__(self)`
```python
def __repr__(self) -> str
```

**Description:** Returns a formatted representation showing the condition and block contents.

**Returns:** String in the format `when {condition} { ... }`.

**Explanation:** Creates a formatted string representation of the conditional block, showing the condition and the block's contents with proper indentation. Uses the parent's `__repr__` method to display the block body.

### `CycledBlock` Class Methods

#### `__init__(self, cycle)`
```python
def __init__(self, cycle: int)
```

**Description:** Creates a cycled block for testbench generation that executes at a specific cycle.

**Parameters:**
- `cycle`: Integer cycle number when the block should execute

**Explanation:** Initializes a cycled block by calling the parent constructor with `Block.CYCLE` kind and stores the cycle number. This type of block is used specifically for testbench generation to schedule operations at precise timing points.

#### `__repr__(self)`
```python
def __repr__(self) -> str
```

**Description:** Returns a formatted representation showing the cycle number and block contents.

**Returns:** String in the format `cycle {number} { ... }`.

**Explanation:** Creates a formatted string representation of the cycled block, showing the cycle number and the block's contents with proper indentation. Uses the parent's `__repr__` method to display the block body.
