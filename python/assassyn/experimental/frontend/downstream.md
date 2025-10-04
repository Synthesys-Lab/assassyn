# Convergent Downstream for Combinational Logic

This module provides all the support and extentions
to  `@factory(Downstream)` decorator.

The key difference between a `Module` and a `Downstream` is that
inputs are sequential or combinational, respectively.
`Downstream` converges combinational logics, so no async call
happends. Its inner function signature is always empty-argument.

## Exposed Interface

````python
def factory_check_signature(inner: Callable) -> bool:
````
For downstream combinational logic, the inner function should not have any arguments.
This function checks if the inner function signature is empty-argument.

--------

````python
def factory_create(inner: Callable, args: dict[str, Port]) -> Factory[Downstream]:
    '''Create a `Downstream` object from the inner function and arguments.'''
````

- It calls the constructor of the `Downstream` class declared in
  [downstream.py](../../ir/downstream/downstream.py) to create a module object.

## Usage

The example below is Turing-equivalant to the 2-stage pipeline example
in [module.md](./module.md) for the purpose of demonstrating `Downstream`.
Then it relies on the combinational pin exposed by `factory.pin(...)`
to connect the two stages without any stage boundary.

````python

# [driver] --|--> [lhs] ----> [adder]
#     |                         ^
#     |                         |
#     +------|--> [rhs] --------+
# where there is no stage boundary, "|", between adder and lhs/rhs

@factory(Downstream)
def adder_downstream_factory(a: Value, b: Value) -> Factory[Downstream]:
    def adder():
        c = a + b
        log("Adder Downstream: {} + {} = {}", a, b, c)
    return adder

@factory(Module)
def forwarder_factory() -> Factory[Module]:
    def forwarder(x: Port[UInt(32)]):
        x = module.pop_all(True)
        factory.pin(x)
    return forwarder

@factory(Module)
def driver_factory(lhs: Factory[Module], rhs: Factory[Module]) -> Factory[Module]:
    def driver():
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        # cnt[0] is passed to lhs and rhs in sequntial logic
        (lhs << {'x': cnt[0]})()
        (rhs << {'x': cnt[0]})()

def top():
    lhs = forwarder_factory()
    rhs = forwarder_factory()
    driver = driver_factory(lhs, rhs)
    # lhs exposes its forwarded data as pin to a
    # rhs exposes its forwarded data as pin to b
    adder = adder_downstream_factory(lhs.pins[0], rhs.pins[0])
````