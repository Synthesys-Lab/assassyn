# Factory Decorator

This module provides the `@pipeline.factory` decorator
to wrap a function to be a pipeline stage factory.

## Exposed Interface

````python
from assassyn.experimental import pipeline

@pipeline.factory
def my_stage_factory(...) -> PipelineStage:
    def my_stage(...):
        ...
    return my_stage
````

This decorator first enforces all the arguments fed to the decorated function
are the same types as annotated, by iterating over the arguments and checking
their types in `__annotations__`.

Then the returned value of the decorated function is checked to be callable.
This decorator shall call this function to grow the AST of the pipeline stage
declared in [module.py](../../ir/module/module.py).

1. First, it extracts all the arguments in the returned inner function signature.
   - It checks that all the arguments have type annotations.
   - It checks that all the argument types are `Port[<some-type>]`.
   - `<some-type>` must be a subclass of `DataType` declared in [dtype.py](../../ir/dtype.py).
2. Then it creates `Port` objects for each argument, with the same name and type, and put them
   into a dictionary.
3. Then, this dictionary will be fed to the constructor of the `Module` class declared in
   [module.py](../../ir/module/module.py) to create a module object.
4. The module name will be renamed to the inner function name capitalized.
   - Check the inner function's name are the same as the outer factory function's name
     with the `_factory` suffix removed.
   - Because `Driver` module is reserved in the old frontend to be the top module,
     so a factory function named `driver_factory` should create a module named `Driver`.
     For now, we make this compromise to keep compatibility.
5. Call the inner function to grow the AST of the module.
   - Just like the usage of `build()` in the old frontend.
     For now, refer to an example in [test_async_call.py](../../../unit-tests/test_async_call.py).
   - The key point is that when calling the inner function.
     The inner function shall capture all the arguments passed to the outer factory function by closure.
   - It relies on a `module.combinational` decorator to set the current insert point
     to grow the AST.

--------

````python
def pop_all(validate: bool = False);
````

This function is a syntactical sugar to pop all the `Port`,
as a helper to call `Module.pop_all_ports`.