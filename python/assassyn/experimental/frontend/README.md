# New Frontend

The old [frontend](../frontend.py) is still in use.
This new frontend serves as a wrapper to the old one
for a more function-like programming style.

## Features

````python
@pipeline.factory
def driver_factory(adder: pipeline.Stage) -> pipeline.StageFactory:
    def driver():
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        with if_(cnt[0] < UInt(32)(100)):
            # You can do either of the following to bind and call a pipeline stage
            # The first is positional
            # (adder << (cnt[0], cnt[0]))
            # The second is named
            # (adder << {'a': cnt[0], 'b': cnt[0]})

            # NOTE: Just like cin, cout in C++, this syntax has side effects.
            #       `<<` binds the arguments to the stage.
            adder << (cnt[0], cnt[0])
            # This calls the stage that was bound before.
            adder()

    return driver

@pipeline.factory
def adder_factory() -> pipeline.StageFactory:
    def adder(a: Port[UInt(32)], b: Port[UInt(32)]):
        a, b = pipeline.pop_all(True)
        c = a + b
        log("Adder: {} + {} = {}", a, b, c)
        return stage.this()
    return adder

sys = SysBuilder('driver')
with sys:
    adder = adder_factory() # Factory w/ empty Stage body
    adder = adder()         # Stage w/ filled Stage body
    driver_factory(adder)()
````

To explain, the new frontend still adopts the concept of trace-based
language implementation, which executes each statement to grow the
AST tree. The overloaded operators will be appended to the existing block.

- `@pipeline.factory` is similar to `@module.combinational` decorator
   in the old frontend, which sets the current block to grow the AST.
     - It is required that a function annotated with `@pipeline.factory`
       should return a function reference.
     - This function reference will be built into a pipeline stage by
       implicitly calling this returned function to grow the AST.
       It is analogous to calling `build` in the old frontend.
       Refer to [../../unit-test/test_async_call.py] for an example.
     - The returned value is not the `Module` object itself,
       but a `Stage` object that wraps the `Module` object.
- `if_` is the same as `Condition` implemented in [block.py](../../ir/block.py).
  Using `if_` better reminds developers that this is a conditional block.

## Implementation

- `@pipeline.factory` decorator is implemented in [pipeline.py](./pipeline.py).
- `@converge.downstream` is implemented in [converge.py](./converge.py).
- `Stage` is implemented in [stage.py](./stage.py).
- For now, we put `if_` in `__init__.py` as a wrapper to `Condition`.