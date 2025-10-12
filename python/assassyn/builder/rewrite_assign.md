# Rewrite Assign

<<<<<<< HEAD
Because assignment in Python cannot be overloaded we want to rewrite the assignment statement with our own function call to hook up the assignment behavior.

## Exposed Functions
=======
This module provides AST transformation functionality to rewrite Python assignment statements
to use a custom assignment function that can be hooked for tracing or other purposes.
Since Python assignment cannot be overloaded, this module uses AST rewriting to intercept
assignment operations and delegate them to the naming system.

## Section 1. Exposed Interfaces

### rewrite_assign

```python
def rewrite_assign(target: ast.FunctionDef) -> ast.FunctionDef:
```

Rewrites assignment statements in a function to use `__assassyn_assignment__`.

This function takes a function definition AST node and transforms all simple identifier 
assignments (e.g., `x = 5`) into calls to `__assassyn_assignment__` 
(e.g., `x = __assassyn_assignment__("x", 5)`).

Assignments to attributes (`obj.attr = val`) and subscripts (`arr[i] = val`) are not rewritten.

**Parameters:**
- `target`: The function definition AST node to transform

**Returns:**
- The transformed function definition AST node

**Explanation:** This function is used by the module decorator system to intercept assignments within module methods. The transformation allows the naming system to track and name IR values based on their assignment targets. The function uses the `AssignmentRewriter` class internally to perform the AST transformation. The rewritten assignments call `__assassyn_assignment__` which delegates to the active [NamingManager](naming_manager.md) for processing.

### __assassyn_assignment__
>>>>>>> 441bfdd (document)

```python
def __assassyn_assignment__(name: str, value: Any) -> Any:
```

<<<<<<< HEAD
This function takes the identifier name and the value to be assigned,
delegates to the active naming manager (when present) to perform naming, and
returns the value (supporting chained assignments).

--------

```python
def rewrite_assign(func=None, *, adjust_lineno=False) -> callable;
```

Decorator to rewrite assignment statements in a function to use `__assassyn_assignment__`.
This is the primary interface for enabling semantic naming in functions.

Can be used in two ways:
1. As a simple decorator: `@rewrite_assign`
2. With parameters: `@rewrite_assign(adjust_lineno=True)`

The decorator:
- Parses the function's source code into an AST
- Transforms simple identifier assignments (e.g., `x = 5`) to use `__assassyn_assignment__` (e.g., `x = __assassyn_assignment__("x", 5)`)
- Handles namespace injection and compilation
- Returns the rewritten function
- Falls back to the original function if rewriting fails

Assignments to attributes (`obj.attr = val`) and subscripts (`arr[i] = val`) are not rewritten.

This provides a unified, reusable interface for code transformation, consolidating functionality that was previously spread across multiple functions.
=======
Assignment function invoked by rewritten assignments.

Delegates to the active NamingManager (if any) to process assignment-based naming, then returns the value. When no manager is active, it simply returns the value unchanged.

**Parameters:**
- `name`: Identifier name being assigned to
- `value`: The value being assigned

**Returns:**
- The assigned value (to support chained assignments)

**Explanation:** This function serves as the hook point for the AST rewriting system. When assignments like `x = some_expr` are rewritten to `x = __assassyn_assignment__("x", some_expr)`, this function processes the naming through the active [NamingManager](naming_manager.md). The function is injected into the namespace of rewritten functions and called during assignment execution. It delegates to `NamingManager.process_assignment()` which applies semantic naming based on the assignment target.

## Section 2. Internal Helpers

### AssignmentRewriter

```python
class AssignmentRewriter(ast.NodeTransformer):
```

AST transformer that rewrites assignments to identifiers.

**Explanation:** This class extends `ast.NodeTransformer` to traverse and modify Python AST nodes. It specifically targets `ast.Assign` nodes and rewrites them to use the `__assassyn_assignment__` function. The transformer only modifies assignments to simple identifiers (Name nodes), leaving attribute assignments and subscript assignments unchanged to avoid breaking object-oriented code patterns.

#### visit_Assign

```python
def visit_Assign(self, node: ast.Assign) -> ast.Assign:
```

Visit an assignment node and rewrite it if it's a simple identifier assignment.

Only rewrites assignments to simple identifiers (Name nodes), not attributes or subscripts.

**Parameters:**
- `node`: The assignment AST node to process

**Returns:**
- The modified assignment node (if it was a simple identifier assignment) or the original node (if not)

**Explanation:** This method implements the core transformation logic. It first visits child nodes using `generic_visit()`, then checks if the assignment target is a single `ast.Name` node. If so, it creates a new assignment where the value is wrapped in a call to `__assassyn_assignment__` with the identifier name as the first argument and the original value as the second argument. This transformation preserves the original assignment semantics while adding the naming hook.
>>>>>>> 441bfdd (document)
