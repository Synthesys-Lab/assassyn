# Convergent Downstream Combinational Logic

This module provides the interfaces to define combinational logic
for module convergence, which serves as the similar purpose as
`@downstream.combinational` in [downstream.py](../../ir/module//downstream.py).

This module is renamed to `converge`, because the combinational logic
in this module is specifically designed for convergence across multiple
modules.

## Usage

```python
from assassyn.experimental.frontend import comb
@comb.converge
def downstream_factory(a: Value, b: Value) -> Downstream:
    def downstream():
        c = a + b
        log("Downstream: {} + {} = {}", a, b, c)
    return downstream
```

Similar as the `@pipeline.factory` decorator, the `@comb.downstream` decorator
has the following constraints:
- The inner function has to have the same name as the outer function
  with `_factory` suffix removed.
  - This instantiates a `Downstream` object with the same name as the
    factory function prefix.
- The inner function should be returned by the outer function.
  - This is required because the outer function is called to grow the AST
    when the factory function is called.
  - The returned inner function is called to grow the AST of the
    combinational logic by the decorator wrapper.
- Finally, the inner function should not have any arguments.
  - This is actually a syntactical salt to force the users to consider
    that every time calling this factory function, a new instance of
    the downstream combinational logic is created.