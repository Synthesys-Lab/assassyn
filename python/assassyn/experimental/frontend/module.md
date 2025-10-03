# Module

This module provides all the support and extentions
to  `@factory(Module)` decorator.

## Exposed Interface

````python
def factory_check_signature(inner: Callable) -> bool:
````
- It checks that all the arguments have type annotations.
  - If a `StageFactory` is passed as an argument expecting `Stage`,
    the underlying `stage` attribute is unwrapped to the factory function.
- It checks that all the argument types are `Port[<some-type>]`.
- `<some-type>` must be a subclass of `DataType` declared in [dtype.py](../../ir/dtype.py).

--------

````python
def factory_create(inner: Callable, args: dict[str, Port]) -> Factory[Module]:
    '''Create a `Module` object from the inner function and arguments.'''
````
- It calls the constructor of the `Module` class declared in
  [module.py](../../ir/module/module.py) to create a module object.
- The name of this module will be renamed by `factory` decorator
  later with unique and capitalized name.

## Extensions

````python
class ModuleFactory(Factory):
    bind: Bind | None # The bind from ir/expr.py

    def __lshift__(self, args: tuple[Value] | dict[str, Value] | Value);
````


**Usage:** This overload is both syntactical sugar and a syntactical salt to remind users
that this function call is different from calling a normal Python function.
We overload the `<<` operator to pass arguments to the function,
and the `()` operator to invoke the function.
Both kw-based and positional argument passing are supported.

````python
# kw-based argument passing
(adder << {'a': a, 'b': b})()
# positional argument passing
(adder << (a, b))()
# continuous argument passing
(adder << a << b)()
````

Even though the target stage has empty input ports, users are still required to
first bind it emptily with `<< {}` and then invoking it with `()`.

````python
# no argument passing
(empty << {})()
````

**Implementation:**
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

--------

````python
# NOTE: This method is not a member of ModuleFactory
def pop_all(validate: bool = False);
````

This function is a syntactical sugar to pop all the `Port`,
as a helper to call `Module.pop_all_ports`.

## Usage
````python
from assassyn.experimental import pipeline

@pipeline.factory
def my_stage_factory(...) -> PipelineStage:
    def my_stage(...):
        ...
    return my_stage

def top():
    stage = my_stage_factory(...)
    stage(...)  # Call the stage 
````
--------

## Usage

This interface provides a more higher-order function flavor of programming.
The example below shows a simple 2-stage pipeline.
- The 1st stage is a self-incrementing counter.
- The 2nd stage is a simple adder, which accepts two inputs from the 1st stage.

````python
# [driver] --|-> [adder]
# where | indicates stage boundary, as well as stage register

@factory(Module)
def driver_factory(adder: Factory[Module]) -> Factory[Module]:
    def driver():
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        with if_(cnt[0] < UInt(32)(100)):
            (adder << cnt[0] << cnt[0])()
    return driver

@factory(Module)
def adder_factory() -> Factory[Module]:
    def adder(a: Port[UInt(32)], b: Port[UInt(32)]):
        a, b = module.pop_all(True)
        c = a + b
        log("adder: {} + {} = {}", a, b, c)
    return adder
  
def top():
    adder = adder_factory()
    driver = driver_factory(adder)

# We still adopt the old SysBuilder context manager
sys = SysBuilder('driver')
with sys:
    top()
````