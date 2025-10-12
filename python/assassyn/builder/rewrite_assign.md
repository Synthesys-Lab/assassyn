# Rewrite Assign

Because assignment in python cannot be overloaded we want to rewrite the assignment statement with our own function call to hook up the assignment behavior.

## Exposed Functions

```python
def rewrite_assign(target: ast.FunctionDef) -> ast.FunctionDef;
```

This function takes a function definition as input, and rewrites the assignment
to identifiers in the function body to calls to `__assassyn_assignment__`.
It uses `ast.NodeTransformer` to traverse and modify the AST.

--------

```python
def __assassyn_assignment__(name: str, value: Any) -> Any;
```

This function takes the identifier name and the value to be assigned,
delegates to the active naming manager (when present) to perform naming, and
returns the value (supporting chained assignments).

--------

```python
def parse_and_rewrite_function(func, adjust_lineno: bool = False) -> tuple;
```

Helper function that extracts the common pattern of parsing a function's source code,
rewriting assignments with AST transformation, and optionally adjusting line numbers.
Returns a tuple of (rewritten_tree, original_lineno).

--------

```python
def named(func) -> callable;
```

Decorator to enable semantic naming for assignments in standalone functions.
Applies AST rewriting to the decorated function and preserves function metadata.
Falls back to the original function if rewriting fails.
