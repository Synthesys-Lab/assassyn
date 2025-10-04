# Visitor Module

This module implements the visitor pattern for traversing the assassyn frontend AST.

---

## Visitor Class

**Attribute:**
- `current_module`: Tracks the module being visited (set for regular modules, `None` for arrays and downstreams)

**Methods:**
- `visit_system(node)`: Entry point. Traverses arrays → modules (with `current_module` set) → downstreams (with `current_module` cleared)
- `visit_module(node)`: Delegates to `visit_block(node.body)`
- `visit_block(node)`: Iterates elements and calls `dispatch()` for each
- `dispatch(node)`: Routes `Expr` to `visit_expr()` and `Block` to `visit_block()`
- `visit_array()`, `visit_expr()`, `visit_port()`: Empty hooks for subclasses to override
