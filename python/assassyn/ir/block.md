# Block Module
The `block.py` module defines the `Block` class hierarchy for representing control flow blocks in the Assassyn IR, along with frontend APIs for creating conditional and cycled blocks.

```python
class Block:
    kind: int                                         # Kind of block (MODULE_ROOT, CONDITIONAL, CYCLE, SRAM)
    _body: list[Expr]                                # List of instructions in the block
    parent: typing.Union[typing.Self, ModuleBase]    # Parent block
    module: typing.Optional[ModuleBase]              # Module of this block
```

## Block Types
The module defines several block kinds as class constants:
- `MODULE_ROOT = 0` - Root block of a module
- `CONDITIONAL = 1` - Conditional execution block
- `CYCLE = 2` - Cycle-based block for testbench generation
- `SRAM = 3` - SRAM-related block for memory block operations in hardware designs

## Exposed Interface
The `block.py` module provides the `Condition` and `Cycle` functions for creating specialized blocks, along with block management methods.

### `Condition(cond)`
```python
@ir_builder(node_type='expr')
def Condition(cond: Value) -> CondBlock  # Frontend API for creating a conditional block
```
- **Description:** Creates a conditional block that executes when the condition is true.
- **Parameters:**
  - `cond`: A `Value` representing the condition to evaluate.
- **Returns:** `CondBlock` instance.
- **Example:**
  ```python
  with Condition(enable_signal):
      # Instructions execute when enable_signal is true
      output.next = input_data
  ```

-------

### `Cycle(cycle)`
```python
@ir_builder(node_type='expr')
def Cycle(cycle: int) -> CycledBlock  # Frontend API for creating a cycled block
```
- **Description:** Creates a cycled block for testbench generation that executes at a specific cycle.
- **Parameters:**
  - `cycle`: Integer cycle number when the block should execute.
- **Returns:** `CycledBlock` instance.
- **Example:**
  ```python
  with Cycle(10):
      # Instructions execute at cycle 10
      test_signal.next = UInt(1, 1)
  ```

-------

## Block Class Methods

### `__init__(self, kind)`
```python
def __init__(self, kind: int)  # Initialize a block with specified kind
```
- **Description:** Creates a new block of the specified kind with empty body.
- **Parameters:**
  - `kind`: Integer constant defining the block type.

-------

### `body` Property
```python
@property
def body(self) -> list[Expr]  # Get the body of the block
```
- **Description:** Returns the list of instructions contained in the block.
- **Returns:** List of `Expr` objects representing the block's instructions.

-------

### `as_operand(self)`
```python
def as_operand(self) -> str  # Dump the block as an operand
```
- **Description:** Returns a string representation of the block for use as an operand.
- **Returns:** String in the format `_{namified_identifier}`.

-------

### `insert(self, x, elem)`
```python
def insert(self, x: int, elem: Expr)  # Insert an instruction at the specified position
```
- **Description:** Inserts an instruction at the given position in the block's body.
- **Parameters:**
  - `x`: Integer position where to insert the instruction.
  - `elem`: `Expr` object to insert.

-------

### `iter(self)`
```python
def iter(self)  # Iterate over the block
```
- **Description:** Generator that yields each instruction in the block's body.
- **Yields:** `Expr` objects from the block's body.

-------

### Context Manager Methods

### `__enter__(self)`
```python
def __enter__(self) -> Block  # Designate the scope of entering the block
```
- **Description:** Sets up the block context when entering a `with` statement. Establishes parent-child relationships by assigning the current block/module as parent, sets the module reference, and switches the builder context with assertions for safe nesting.
- **Returns:** The block instance for use in the `with` statement.

-------

### `__exit__(self, exc_type, exc_value, traceback)`
```python
def __exit__(self, exc_type, exc_value, traceback)  # Designate the scope of exiting the block
```
- **Description:** Cleans up the block context when exiting a `with` statement.
- **Parameters:** Standard exception handling parameters.

-------

### `__repr__(self)`
```python
def __repr__(self) -> str  # String representation with proper indentation
```
- **Description:** Returns a formatted string representation of the block with proper indentation for nested structures.
- **Returns:** Indented string showing the block's contents.

-------

## CondBlock Class
**Inherits from:** `Block`

### `__init__(self, cond)`
```python
def __init__(self, cond: Value)  # Initialize conditional block with condition
```
- **Description:** Creates a conditional block that executes when the condition is true.
- **Parameters:**
  - `cond`: `Value` representing the condition to evaluate.

-------

### `__repr__(self)`
```python
def __repr__(self) -> str  # String representation of conditional block
```
- **Description:** Returns a formatted representation showing the condition and block contents.
- **Returns:** String in the format `when {condition} { ... }`.

-------

## CycledBlock Class
**Inherits from:** `Block`

### `__init__(self, cycle)`
```python
def __init__(self, cycle: int)  # Initialize cycled block with cycle number
```
- **Description:** Creates a cycled block for testbench generation that executes at a specific cycle.
- **Parameters:**
  - `cycle`: Integer cycle number when the block should execute.

-------

### `__repr__(self)`
```python
def __repr__(self) -> str  # String representation of cycled block
```
- **Description:** Returns a formatted representation showing the cycle number and block contents.
- **Returns:** String in the format `cycle {number} { ... }`.
