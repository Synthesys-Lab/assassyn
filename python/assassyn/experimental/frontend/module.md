# Module Factory Support

This module provides factory support functions for the `@factory(Module)` decorator
in the experimental frontend. Module instances represent pipeline stages in
Assassyn's credit-based pipeline architecture, implementing sequential logic
with stage boundaries and async communication.

## Summary

Module instances are the primary building blocks of Assassyn's credit-based pipeline
architecture as described in [arch.md](../../../docs/design/arch/arch.md). Unlike
`Downstream` modules that implement combinational logic, `Module` instances operate
sequentially with explicit stage boundaries, enabling async communication between
pipeline stages through the credit system.

Modules receive data through input ports and can make async calls to other modules,
consuming credits in the process. This enables the credit-based flow control
mechanism where modules wait for credits before executing.

## Exposed Interfaces

### factory_check_signature

```python
def factory_check_signature(inner: Callable[..., Any]) -> Dict[str, Port]:
    """Validate inner signature and synthesise module ports."""
```

**Purpose**: Validates the inner function signature and creates port definitions
for the module based on parameter annotations.

**Parameters**:
- `inner`: The inner function to validate

**Returns**: Dictionary mapping parameter names to `Port` instances

**Raises**: `TypeError` if parameters lack type annotations or have invalid types

**Explanation**: This function validates that all parameters in the inner function
have type annotations of the form `Port[DataType]`. It creates `Port` instances
for each parameter, which will become the module's input ports. The ports enable
the module to receive data from upstream stages through the credit-based pipeline
system.

### factory_create

```python
def factory_create(_inner: Callable[..., Any], args: Dict[str, Port]) -> Tuple[Module, Dict[str, Port]]:
    """Instantiate a Module and prepare kwargs for the inner builder."""
```

**Purpose**: Creates a new `Module` instance and prepares keyword arguments for
the inner function execution.

**Parameters**:
- `_inner`: The inner function (unused in module creation)
- `args`: Dictionary of port definitions

**Returns**: Tuple of (module instance, keyword arguments for inner function)

**Explanation**: Creates a `Module` instance using the port definitions and
prepares keyword arguments that map port names to their corresponding port
objects. This enables the inner function to access its input ports by name
during execution.

### pop_all

```python
def pop_all(validate: bool = False):
    """Pop all ports from the current module under construction."""
```

**Purpose**: Syntactic sugar for popping all input ports from the current module.

**Parameters**:
- `validate`: Whether to validate all ports have data before popping

**Returns**: List of popped values (or single value if only one port)

**Raises**: `RuntimeError` if called outside an active module context

**Explanation**: This function provides convenient access to all input ports
of the current module. When `validate=True`, it ensures all ports have valid
data before popping, setting the module to backpressure timing mode.
Otherwise, it uses systolic timing mode.

## Internal Helpers

### ModuleFactory

```python
class ModuleFactory(Factory[Module]):
    """Wrapper around `Module` providing bind/call sugar."""
```

**Purpose**: Specialized factory wrapper for `Module` instances that provides
syntactic sugar for binding arguments and making async calls.

**Attributes**:
- `bind`: Optional `Bind` instance for argument binding

**Methods**:

#### __lshift__

```python
def __lshift__(self, args):
    """Bind arguments to the module using the << operator."""
```

**Purpose**: Provides syntactic sugar for binding arguments to module ports.

**Parameters**:
- `args`: Arguments to bind (Value, tuple, or dict)

**Returns**: Self for method chaining

**Raises**: `ValueError` if too many positional arguments provided
**Raises**: `TypeError` if arguments are not Value, tuple, or dict

**Explanation**: This operator overload enables convenient argument binding
using the `<<` operator. It supports:
- Single `Value` objects (converted to single-element tuples)
- Tuples of `Value` objects (bound to unbound ports in order)
- Dictionaries mapping port names to `Value` objects

The operator creates a `Bind` instance if none exists and binds arguments
to the module's ports. For positional arguments, it automatically maps
to unbound ports in order.

#### __call__

```python
def __call__(self):
    """Make an async call to the bound module."""
```

**Purpose**: Executes an async call to the module using bound arguments.

**Returns**: `AsyncCall` expression

**Raises**: `ValueError` if no arguments have been bound

**Explanation**: Creates an async call to the module using the arguments
bound via the `<<` operator. This implements the credit-based communication
pattern where the caller increases the callee's credit counter and the
callee consumes credits when executing.

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