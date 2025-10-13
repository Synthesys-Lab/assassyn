# Rewrite Assign

This module provides AST transformation functionality to rewrite Python assignment statements
to use a custom assignment function that can be hooked for tracing or other purposes.
Since Python assignment cannot be overloaded, this module uses AST rewriting to intercept
assignment operations and delegate them to the naming system.

## Section 0. Summary

The rewrite_assign module enables semantic naming in Assassyn by transforming Python assignment statements at the AST level. When assignments like `x = some_expr` are rewritten to `x = __assassyn_assignment__("x", some_expr)`, the naming system can track and name IR values based on their assignment targets. This is essential for generating meaningful names in the generated code.

## Section 1. Exposed Interfaces

### `rewrite_assign`

```python
def rewrite_assign(func=None, *, adjust_lineno=False) -> callable:
```

Decorator to rewrite assignment statements in a function to use `__assassyn_assignment__`.
This is the primary interface for enabling semantic naming in functions.

Can be used in two ways:
1. As a simple decorator: `@rewrite_assign`
2. With parameters: `@rewrite_assign(adjust_lineno=True)`

**Parameters:**
- `func`: The function to rewrite (when used as @rewrite_assign)
- `adjust_lineno`: If True, adjust AST line numbers to match original source location

**Returns:**
- The rewritten function (or decorator if called with parameters)

**Explanation:** This decorator implements a sophisticated AST transformation pipeline. It first parses the function's source code using `inspect.getsource()` and `ast.parse()`, then applies the `AssignmentRewriter` transformer to convert simple identifier assignments (e.g., `x = 5`) into calls to `__assassyn_assignment__` (e.g., `x = __assassyn_assignment__("x", 5)`). The decorator handles namespace injection by temporarily adding the `__assassyn_assignment__` function to the function's global namespace, compiles the transformed AST, and preserves function metadata using `functools.wraps`. If AST rewriting fails for any reason, it gracefully falls back to the original function. The `adjust_lineno` parameter allows preserving original source line numbers for better error reporting and debugging.

### `__assassyn_assignment__`

```python
def __assassyn_assignment__(name: str, value: Any) -> Any:
```

Assignment function invoked by rewritten assignments.

Delegates to the active NamingManager (if any) to process assignment-based naming, then returns the value. When no manager is active, it simply returns the value unchanged.

**Parameters:**
- `name`: Identifier name being assigned to
- `value`: The value being assigned

**Returns:**
- The assigned value (to support chained assignments)

**Explanation:** This function serves as the hook point for the AST rewriting system. When assignments like `x = some_expr` are rewritten to `x = __assassyn_assignment__("x", some_expr)`, this function processes the naming through the active [NamingManager](naming_manager.md). The function is injected into the namespace of rewritten functions and called during assignment execution. It delegates to `NamingManager.process_assignment()` which applies semantic naming based on the assignment target. The function returns the original value to support chained assignments like `a = b = c`.

## Section 2. Internal Helpers

### `AssignmentRewriter`

```python
class AssignmentRewriter(ast.NodeTransformer):
```

AST transformer that rewrites assignments to identifiers.

**Explanation:** This class extends `ast.NodeTransformer` to traverse and modify Python AST nodes. It specifically targets `ast.Assign` nodes and rewrites them to use the `__assassyn_assignment__` function. The transformer only modifies assignments to simple identifiers (Name nodes), leaving attribute assignments and subscript assignments unchanged to avoid breaking object-oriented code patterns.

#### `visit_Assign`

```python
def visit_Assign(self, node: ast.Assign) -> ast.Assign:
```

Visit an assignment node and rewrite it if it's a simple identifier assignment.

Only rewrites assignments to simple identifiers (Name nodes), not attributes or subscripts.

**Parameters:**
- `node`: The assignment AST node to process

**Returns:**
- The modified assignment node (if it was a simple identifier assignment) or the original node (if not)

**Explanation:** This method implements the core transformation logic. It first visits child nodes using `generic_visit()`, then checks if the assignment target is a single `ast.Name` node. If so, it creates a new assignment where the value is wrapped in a call to `__assassyn_assignment__` with the identifier name as the first argument and the original value as the second argument. This transformation preserves the original assignment semantics while adding the naming hook. The method specifically avoids rewriting attribute assignments (`obj.attr = val`) and subscript assignments (`arr[i] = val`) to prevent interference with object-oriented programming patterns and array operations.