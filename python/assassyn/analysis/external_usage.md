# External Usage Analysis

This module provides utilities for analyzing external usage patterns of expressions and operands across different modules in the Assassyn IR.

## Section 1. Exposed Interfaces

### `get_module(operand: Operand) -> ModuleBase | None`

```python
def get_module(operand: Operand) -> ModuleBase | None:
    """Get the module that contains the given operand."""
```

**Parameters:**
- `operand`: The operand whose containing module needs to be determined

**Returns:**
- `ModuleBase | None`: The module that contains the operand, or `None` if the operand's user is not an `Expr`

**Behavior:**
This function determines which module contains a given operand by examining the operand's user. If the user is an `Expr`, it returns the module recorded on the expression's `parent` field. Otherwise, it returns `None`.

### `expr_externally_used(expr: Expr, exclude_push: bool) -> bool`

```python
def expr_externally_used(expr: Expr, exclude_push: bool) -> bool:
    """Check if an expression is used outside its module."""
```

**Parameters:**
- `expr`: The expression to check for external usage
- `exclude_push`: If `True`, `FIFOPush` expressions are excluded from external usage analysis

**Returns:**
- `bool`: `True` if the expression is used outside its containing module, `False` otherwise

**Behavior:**
This function analyzes whether an expression is used by other modules outside its own module. It iterates through all users of the expression and checks if any user belongs to a different module than the expression's owning module. When `exclude_push` is `True`, `FIFOPush` expressions are automatically considered as not externally used.

## Section 2. Internal Helpers

This module does not contain internal helper functions. All functionality is exposed through the two main functions described above.

## Usage Context

### Code Generation Integration

The `expr_externally_used` function is primarily used in code generation phases to determine whether expressions need to be exposed as module interfaces:

1. **Simulator Code Generation** (`codegen/simulator/modules.py`): Used to determine if a valued expression needs exposure when generating simulator modules.

2. **Verilog Code Generation** (`codegen/verilog/design.py`): Used to determine if expressions should be exposed in the generated Verilog design, particularly for non-constant expressions that are used externally.

### Module Analysis

The functions work together to analyze cross-module dependencies and usage patterns, which is essential for:
- Determining module interface requirements
- Optimizing code generation by identifying expressions that need to be exposed
- Understanding data flow across module boundaries

## Dependencies

This module depends on the following IR components:
- `Expr`, `Operand`, and `FIFOPush` from `ir.expr`
- `ModuleBase` from `ir.module.base`

## Technical Notes

The analysis is based on the IR's user-definer relationships, where each expression maintains a list of its users (operands that reference it). The module containment is determined through the parent-child relationships in the IR structure.
