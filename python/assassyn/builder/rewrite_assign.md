# Rewrite Assign

Because assignment in Python cannot be overloaded we want to rewrite the assignment statement with our own function call to hook up the assignment behavior.

## Exposed Functions

```python
def __assassyn_assignment__(name: str, value: Any) -> Any;
```

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
