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
    def __lshift__(self, args: tuple | dict);
````

This operator overloads the `<<` operator to bind arguments to the stage.
It takes either a tuple or a dictionary as input.
If the bind node is `None`, it creates a new `Bind` node upon first call and put it in `self.bind`.
If the bind already exists, the created value bind will be updated to the `self.bind` node.

--------

````python
    def __call__(self);
````

This operator creates a async call to the bind, which serves as the similar purpose as
`Module.async_called` in the old frontend. Call is always `void` argument, as arguments
are fed by bindings.