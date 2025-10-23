# Base Module

This file provides the foundational components for defining hardware modules, including a base class and a decorator factory for combinational logic.

## Design Documents

- [Module Design](../../../docs/design/internal/module.md) - Module design and lifecycle
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture
- [DSL Design](../../../docs/design/lang/dsl.md) - Trace-based DSL system

## Related Modules

- [Module Implementation](../module.md) - Main module implementation
- [Downstream Module](../downstream.md) - Downstream module implementation
- [Rewrite Assign](../../builder/rewrite_assign.md) - AST rewriting system
- [Naming Manager](../../builder/naming_manager.md) - Naming system

## Section 0. Summary

The `base.py` module implements the core infrastructure for Assassyn's module system. It provides `ModuleBase`, the parent class for all hardware modules that enables external dependency tracking and operand representation. Additionally, it implements `combinational_for`, a decorator factory that creates specialized decorators for module logic functions, handling IR context management and automatic signal naming through AST rewriting. This module is essential for the [credit-based pipeline architecture](../../../docs/design/arch/arch.md) as it provides the base functionality that all module types inherit.

## Section 1. Exposed Interfaces

### `ModuleBase`

```python
class ModuleBase:
    def __init__(self) -> None
    def as_operand(self) -> str
    def triggered(self) -> PureIntrinsic
    def add_external(self, operand: Operand) -> None
    @property
    def externals(self) -> typing.Dict[Expr, typing.List[Operand]]
```

The base class for all hardware module definitions in Assassyn. This class provides core functionality for dependency tracking, operand representation, and module introspection that all module types inherit.

#### `as_operand`

```python
def as_operand(self) -> str:
    '''Dump the module as a right-hand side reference.'''
```

Returns a unique string identifier for the module when it is referenced as an operand in expressions. This identifier is used throughout the IR for debugging and code generation purposes.

**Explanation:** This method provides a consistent way to reference modules in the generated IR. It first checks for the module's `name` attribute and falls back to generating a unique identifier using the module's object identity. The generated name follows the pattern `_{namified_identifier}` to distinguish module references from other operands. This is essential for [Verilog code generation](../../../docs/design/internal/pipeline.md) where modules need unique identifiers for instantiation.

#### `triggered`

```python
@ir_builder
def triggered(self) -> PureIntrinsic:
    '''The frontend API for creating a triggered node,
    which checks if this module is triggered this cycle.
    NOTE: This operation is only usable in downstream modules.'''
```

Creates an intrinsic that checks if this module was activated in the current simulation cycle. This is primarily used in downstream modules to implement conditional logic based on upstream module activation.

**Usage Context:** The `triggered()` method is designed for use in downstream modules to check if their upstream dependencies were activated. While the documentation states it's "only usable in downstream modules," the current implementation doesn't enforce this restriction at runtime. This is a design consideration that may be addressed in future versions.

**Explanation:** This method creates a `PureIntrinsic` with `MODULE_TRIGGERED` opcode that represents the module's activation status. In the [credit-based pipeline architecture](../../../docs/design/internal/pipeline.md), modules are activated when they have credits available. This intrinsic allows downstream modules to conditionally execute logic based on whether their upstream dependencies were active. The method is decorated with `@ir_builder` to ensure proper IR construction and context management.

#### `add_external`

```python
def add_external(self, operand: Operand) -> None:
    '''Add an external operand to this module.'''
```

Records external dependencies by adding operands that reference values from other modules or arrays. This is crucial for building the system dependency graph.

**External Dependency Detection Logic:** The method determines if an operand is "external" by examining its value:

1. **Array References**: If the operand references an `Array` object, it's considered external
2. **Module References**: If the operand references a `Module` object, it's considered external  
3. **Cross-Module Expression References**: If the operand references an expression from a different module, it's considered external
4. **Block Condition References**: Special handling for `CondBlock` conditions that may reference external values

The detection logic is complex and handles various edge cases with nested expressions and complex operand chains. External dependencies are stored in the `_externals` dictionary and used during code generation to establish proper module connections.

**Explanation:** This method implements external dependency tracking, which is essential for [module generation](../../../docs/design/internal/module.md). It examines the operand's value to determine if it references external resources (other modules, arrays, or expressions from different modules). External dependencies are stored in `_externals` dictionary and used during code generation to establish proper module connections. This tracking ensures that the generated hardware correctly connects modules based on their actual usage patterns.

### `combinational_for`

```python
def combinational_for(module_type) -> Callable:
    '''Decorator factory for combinational module build functions with naming support.'''
```

A decorator factory that creates specialized decorators for module logic functions. The returned decorator handles IR context management and automatic signal naming through AST rewriting.

**Parameter Binding Rules:** The decorator implements sophisticated parameter binding logic:

1. **Array Object Binding**: For `Array` objects, the decorator preserves hierarchical names with underscores, except for the 'array_xxxxx' pattern which is simplified
2. **Module Attribute Binding**: Function parameters are automatically bound to module attributes with the same name
3. **Type Preservation**: The binding maintains the original object types while creating the necessary connections
4. **Hierarchical Naming**: Complex naming patterns are preserved to maintain semantic meaning in the generated hardware

**Explanation:** This decorator factory implements sophisticated module logic definition capabilities. It creates a decorator that wraps module build functions with several key features:

1. **IR Context Management**: Automatically enters and exits the module's context in the global IR builder, ensuring all generated IR nodes are properly associated with the module.

2. **AST Rewriting**: Uses the [rewrite_assign](../../builder/rewrite_assign.md) decorator to transform Python assignment statements, enabling automatic signal naming based on variable names in the source code.

3. **Parameter Binding**: Automatically binds function parameters to module attributes, enabling clean parameter passing from the module interface to the logic implementation.

4. **Error Handling**: Gracefully handles AST rewriting failures by falling back to the original function, ensuring robustness.

The decorator is essential for the [DSL abstraction](../../../docs/design/lang/dsl.md) as it bridges the gap between Python function definitions and hardware module logic, providing the syntactic sugar that makes Assassyn's module definitions intuitive and debuggable.

## Section 2. Internal Helpers

### `_dump_externals`

```python
def _dump_externals(self) -> str:
```

Internal helper method that generates a string representation of all external dependencies for debugging purposes.

**Explanation:** This method iterates through the `_externals` dictionary and creates a formatted string showing all external dependencies and their usage contexts. It handles different operand types (including `Block` conditions) and provides detailed information about where each external dependency is used. This is primarily used in module string representations for debugging and IR inspection.
