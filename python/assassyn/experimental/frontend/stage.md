# Stage, Pipeline Stage

A `Stage` is a pipeline stage that wraps a `Module` object declared in
[module.py](../../ir/module/module.py).

## Exposed Interface

````python
class Stage:
    m: Module # The wrapped module
    bind: Bind # The bind from ir/expr.py
````

--------

````python
    def __init__(self, module: dict[str, Port], name: str);
````
The constructor takes a dictionary of ports that maps port names to `Port` objects,
as well as a name for the stage. This dictionary shall be created in the
`@pipeline.factory` decorator declared in [pipeline.py](./pipeline.py).

This constructor first creates the wrapped `Module` object with the given ports.
Then, it renames the module name to the given name.

--------

````python
    def __lshift__(self, args: tuple[Value] | dict[str, Value] | Value);
````

If `self.bind` is `None`, this operator calls `self.m.bind()` to create an empty first `Bind`.
Then, then operator overloads the `<<` operator to bind arguments to the stage.
- A single `Value` bind is converted to a single-element tuple bind (see below).
- If the `args` is `Tuple[Value]`, it pushes value bindings to unbound ports in order.
  This should be done by converting positional to kw arguments, because `self.bind` provides
  onlyy `**kwargs` interface:
    1. traversing `self.bind.pushes` (declared in [call.py](../../ir/expr/call.py))
    to find all unbound ports.
    2. map the first `len(args)` unbound ports to the given `args`.
- If the `args` is a dictionary mapping port names to `Value` objects, it
  binds the values to the corresponding ports.

--------

````python
    def __call__(self);
````

This operator creates a async call to the bind by calling `self.bind.async_called()` in the old frontend.
Call is always `void` argument, as arguments are fed by bindings.