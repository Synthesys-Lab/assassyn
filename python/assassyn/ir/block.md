# Block Module

## Design Documents

- [DSL Design](../../../docs/design/lang/dsl.md) - Trace-based DSL system and block-based control flow
- [Module Design](../../../docs/design/internal/module.md) - Module generation and control flow
- [Simulator Design](../../../docs/design/internal/simulator.md) - Simulator design and testbench generation
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system

## Related Modules

- [Builder Singleton](../../builder/__init__.md) - Builder context management system
- [Expression Base](../expr.md) - Base expression classes
- [Module Base](../module/base.md) - Base module functionality

## Section 0. Summary

The `block.py` module defines the `Block` class hierarchy for representing control flow blocks in the Assassyn IR. This module implements the block-based control flow system that works with the [builder singleton](../../builder/__init__.py) to manage the current insertion point for IR nodes. Blocks serve as containers for expressions and provide context management through Python's `with` statement, enabling conditional execution (including cycle-based predicates expressed with `current_cycle()`) as described in the [DSL design](../../../docs/design/dsl.md).

## Section 1. Exposed Interfaces

This section describes all the function interfaces and data structures that are exposed to other parts of the project.

### Data Structures

#### `Block`
```python
class Block:
    kind: int                                         # Kind of block (MODULE_ROOT, CONDITIONAL)
    _body: list[Expr]                                # List of instructions in the block
    parent: typing.Union[typing.Self, ModuleBase]    # Parent block
    module: typing.Optional[ModuleBase]              # Module of this block
```

**Purpose:** Base class for all control flow blocks in the Assassyn IR.

**Member Fields:**
- `kind`: Integer constant defining the block type (MODULE_ROOT, CONDITIONAL)
- `_body`: List of `Expr` objects representing the instructions contained in this block
- `parent`: Reference to the parent block or module that contains this block
- `module`: Reference to the module that owns this block

**Static Member Fields:**
- `MODULE_ROOT = 0`: Constant for root blocks of modules
- `CONDITIONAL = 1`: Constant for conditional execution blocks

**Block Kind Usage Patterns:**

1. **MODULE_ROOT (0)**: Used for the root block of each module. This block contains all the module's logic and serves as the top-level container for expressions. Created automatically when a module is instantiated.

2. **CONDITIONAL (1)**: Used for conditional execution blocks created by the `Condition()` function. These blocks execute their contents only when the specified condition is true, implementing multiplexer-based conditional logic in the generated hardware. Cycle-based gating is expressed by conditions comparing `current_cycle()` to a value (e.g., via `Cycle(N)`), not by a special block kind.

**Note on Block Kind Constants:** These constants are currently defined as integer values. Consider using an enum for better type safety and maintainability in future versions.

#### `CondBlock`
```python
class CondBlock(Block):
    cond: Value  # Condition for this block
```

**Purpose:** Represents conditional blocks that execute when a condition is true.

**Member Fields:**
- `cond`: `Value` object representing the condition that determines when this block executes

####

### Functions

#### `Condition(cond)`
```python
@ir_builder(node_type='expr')
def Condition(cond: Value) -> ContextManager
```

**Description:** Frontend API for conditionally guarding statements using predicate intrinsics. It is now a thin sugar that pushes the predicate on enter and pops it on exit, without constructing a structural `CondBlock`.

**Parameters:**
- `cond`: A `Value` representing the condition to push onto the predicate stack

**Returns:** A context manager that calls `push_condition(cond)` on enter and `pop_condition()` on exit

**Explanation:** `Condition` integrates with the per-module predicate stack managed by the builder's `ModuleContext`. The simulator emits real `if {}` blocks for `push_condition`/`pop_condition`, while the Verilog dumper uses the predicate stack for `get_pred()` gating. Existing code can continue using `with Condition(cond): ...` unchanged.

**Example:**
```python
with Condition(enable_signal):
    log("enabled: {}", enable_signal)
```

**Deprecation note:** The structural `CondBlock` remains available for import-compatibility but is deprecated; new code should rely on `Condition` sugar or explicit `push_condition`/`pop_condition`.

#### `Cycle(cycle)`
```python
@ir_builder(node_type='expr')
def Cycle(cycle: int) -> CondBlock
```

**Description:** Frontend helper for creating a conditional block that triggers at a specific cycle.

**Parameters:**
- `cycle`: Integer cycle number when the block should execute

**Returns:** A `Condition` context manager equivalent to `Condition(current_cycle() == cycle)`

**Explanation:** This function creates a conditional block using a predicate `current_cycle() == cycle`. The block executes at the specified cycle during simulation, allowing testbench logic to be scheduled at precise timing points, without requiring a dedicated block kind. See [simulator design](../../../docs/design/simulator.md) for timing details.

**Example:**
```python
with Cycle(10):
    # Instructions execute when current_cycle() == 10
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
- `kind`: Integer constant defining the block type (MODULE_ROOT, CONDITIONAL)

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

**Explanation:** Implements the context manager protocol for blocks. It fetches the active builder via `Singleton.peek_builder()`, establishes parent-child relationships by assigning the current block/module as parent, sets the module reference from the builder, and switches the builder context to this block. This enables the block to become the current insertion point for new IR nodes. The method includes assertions to ensure safe nesting of blocks.

**Builder Context Management Integration:** The block system integrates with the builder singleton through a context stack mechanism:

1. **Context Entry**: When entering a block via `__enter__()`, the block calls `Singleton.peek_builder().enter_context_of('block', self)` to push itself onto the context stack
2. **Context Exit**: When exiting a block via `__exit__()`, the block calls `Singleton.peek_builder().exit_context_of('block')` to pop itself from the context stack
3. **Nested Blocks**: The context stack allows for proper nesting of blocks, with each block maintaining its own insertion point
4. **Error Handling**: The context management includes assertions to prevent invalid nesting scenarios and ensure proper cleanup

**Error Conditions:**
- `AssertionError`: Raised if block nesting depth exceeds safe limits
- Context management errors: May occur if builder singleton is not properly initialized
- Module reference errors: May occur if module context is not available when entering blocks

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

IR dump special rule for predicate intrinsics

- When predicate intrinsics are present in a block body, the dump renders them as visual braces to make the guarded region obvious while reminding there is no structural control flow.
  - On `push_condition(cond)`, the dump prints a single line:
    - `if cond { // PUSH_CONDITION`
    - The indentation then increases for subsequent lines.
  - On `pop_condition()`, the dump first reduces indentation, then prints a single line:
    - `} // POP_CONDITION`
  - These markers are formatting-only hints for developers; they do not represent a real `if` statement in the IR.
  - Indentation is still managed by `Singleton.repr_ident` and remains balanced by the builder’s predicate stack invariants.

**Global State Management for String Representation:** The `__repr__` methods use `Singleton.repr_ident` for indentation, which creates a global state management pattern:

1. **Global Indentation Counter**: All block types modify the global `Singleton.repr_ident` counter for indentation
2. **Thread Safety Considerations**: This global state approach may not be thread-safe and could cause issues in multi-threaded environments
3. **Nested Block Indentation**: The indentation system works correctly for nested blocks but relies on global state
4. **Alternative Approaches**: Consider using a more localized approach or thread-local storage for better isolation

**Error Conditions:**
- Global state conflicts: May occur if multiple threads access the global indentation counter simultaneously
- Indentation errors: May occur if the global state is not properly managed across nested blocks

### `CondBlock` Class Methods

#### `__init__(self, cond)`
```python
def __init__(self, cond: Value)
```

**Description:** Creates a conditional block that executes when the condition is true.

**Parameters:**
- `cond`: `Value` representing the condition to evaluate

**Explanation:** Initializes a conditional block by calling the parent constructor with `Block.CONDITIONAL` kind. Wraps the condition in an `Operand` object and establishes the user relationship if the condition is an expression. This ensures proper dependency tracking in the IR.

**Operand Wrapping and User Relationship Management:** The `CondBlock.__init__` method implements a specific pattern for managing operand relationships:

1. **Operand Wrapping**: The condition is wrapped in an `Operand` object using `Operand(cond, self)`, creating a directed link between the condition value and the block
2. **User Relationship**: If the condition is an `Expr` object, the block is added to the condition's users list via `cond.users.append(self.cond)`
3. **Dependency Tracking**: This pattern ensures that the IR maintains proper use-def relationships, enabling dataflow analysis and optimization
4. **Consistency**: This pattern should be consistent across all block types that reference external values

**Error Conditions:**
- Type errors: May occur if the condition is not a valid `Value` object
- Dependency tracking errors: May occur if user relationships are not properly established

#### `__repr__(self)`
```python
def __repr__(self) -> str
```

**Description:** Returns a formatted representation showing the condition and block contents.

**Returns:** String in the format `when {condition} { ... }`.

**Explanation:** Creates a formatted string representation of the conditional block, showing the condition and the block's contents with proper indentation. Uses the parent's `__repr__` method to display the block body.

### Migration Note: CycledBlock Removal

`CycledBlock` and `Block.CYCLE` have been removed. Use `Condition(current_cycle() == N)` or the helper `Cycle(N)` which returns a `Condition` context manager.

This change reduces indirection and unifies conditional semantics across compile-time and runtime conditions.

## Section 3. Deprecations

### `CondBlock` (deprecated)

`CondBlock` previously represented structural conditional blocks. Predicate semantics are now implemented via intrinsics. `CondBlock` is retained temporarily for compatibility, but it should not be constructed by `Condition` anymore and may be removed in a future version.
