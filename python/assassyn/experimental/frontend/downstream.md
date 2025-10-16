# Downstream Module Factory Support

This module provides factory support functions for the `@factory(Downstream)` decorator
in the experimental frontend. Downstream modules implement combinational logic that
converges data from multiple pipeline stages without stage boundaries.

## Design Documents

- [Experimental Frontend Design](../../../docs/design/lang/experimental_fe.md) - Experimental frontend architecture
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Module Factory Support](./module.md) - Module factory support
- [Unified Factory Decorator](./factory.md) - Unified factory pattern implementation
- [Downstream Implementation](../../ir/module/downstream.md) - Downstream module implementation
- [Module Implementation](../../ir/module/module.md) - Main module implementation

## Summary

Downstream modules are a special type of module in Assassyn's credit-based pipeline
architecture. Unlike regular `Module` instances that operate sequentially with stage
boundaries, `Downstream` modules implement pure combinational logic that can receive
data from multiple sources in the same cycle. This enables efficient cross-stage
communication patterns as described in the [architectural design](../../../docs/design/arch/arch.md).

**Port vs Pin Distinction:** In the experimental frontend, there's an important distinction between ports and pins:
- **Ports**: Used for sequential communication between pipeline stages (Module instances). Ports have FIFO buffers and enable async communication with credit-based flow control.
- **Pins**: Used for combinational communication without stage boundaries. Pins enable immediate data access across modules without FIFO buffering or credit management.

The key architectural difference is that `Downstream` modules have no input ports
and their inner functions take no arguments, making them purely combinational.

## Exposed Interfaces

### factory_check_signature

```python
def factory_check_signature(inner: Callable[..., Any]) -> bool:
    """Validate that the inner function has no arguments (empty signature)."""
```

**Purpose**: Validates that downstream module inner functions have empty signatures,
as downstream modules are purely combinational and receive data through exposed pins
rather than input ports.

**Parameters**:
- `inner`: The inner function to validate

**Returns**: `True` if validation passes

**Raises**: `TypeError` if the inner function has any parameters

**Explanation**: Downstream modules implement combinational logic that converges data
from multiple sources. Since they operate without stage boundaries, they cannot have
input ports like regular modules. Instead, they receive data through combinational
pins exposed by upstream modules using the `pin()` function. This validation ensures
the inner function signature matches this architectural constraint.

### factory_create

```python
def factory_create(_inner: Callable[..., Any], _args: bool) -> Downstream:
    """Instantiate a Downstream module."""
```

**Purpose**: Creates a new `Downstream` module instance for the factory pattern.

**Parameters**:
- `_inner`: The inner function (unused in downstream creation)
- `_args`: Arguments payload (unused for downstream modules)

**Returns**: A new `Downstream` instance

**Explanation**: Creates a `Downstream` module by calling the constructor of the
`Downstream` class declared in [downstream.py](../../ir/module/downstream.py).
The module name will be assigned by the factory decorator using the naming manager
to ensure uniqueness.

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