# Rewrite Assign

Because assignment in python cannot be overloaded we want to rewrite the assignment statement with our own function call to hook up the assignment behavior.

## Exposed Function

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
