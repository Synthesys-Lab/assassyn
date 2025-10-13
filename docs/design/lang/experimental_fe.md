# Experimental Functional Frontend

This document describes the design of Assassyn's experimental functional frontend,
which provides a higher-order function approach to module construction compared to
the legacy imperative frontend. The experimental frontend enables functional
programming patterns while maintaining compatibility with Assassyn's credit-based
pipeline architecture.

## Overview

The experimental frontend represents a significant architectural evolution from the
legacy frontend, providing a more functional programming style for constructing
modules. Instead of imperative class-based construction, modules are created through
factory functions that return inner module definitions, enabling better composability
and type safety.

### Design Philosophy

The experimental frontend adopts several key design principles:

1. **Functional Composition**: Modules are constructed through higher-order functions
   that return inner module definitions, enabling functional composition patterns.

2. **Type Safety**: The factory pattern provides compile-time type checking through
   Python's type annotations, ensuring module interfaces are correctly specified.

3. **Unified Interface**: A single `@factory` decorator handles different module
   types (Module, Downstream) through a unified interface while delegating
   type-specific validation and construction.

4. **Syntactic Sugar**: Operator overloads (`<<` for binding, `()` for calling)
   provide intuitive syntax for module communication while maintaining the
   credit-based pipeline semantics.

## Architecture

### Factory Pattern

The core of the experimental frontend is the factory pattern implemented through
the `@factory(type)` decorator:

```python
@factory(Module)
def my_module_factory(...) -> Factory[Module]:
    def my_module(...):
        # Module implementation
    return my_module
```

The factory pattern provides several benefits:

- **Automatic AST Construction**: The decorator handles entering/exiting module
  construction contexts automatically
- **Type-specific Validation**: Each module type provides specialized validation
  through `factory_check_signature` and `factory_create` methods
- **Unified Construction**: Common construction logic is shared across module types
- **Naming Management**: Automatic unique naming through the naming manager

### Module Types

The experimental frontend supports two primary module types:

#### Module (Pipeline Stages)

`Module` instances represent pipeline stages in the credit-based pipeline architecture.
They implement sequential logic with explicit stage boundaries and async communication:

```python
@factory(Module)
def adder_factory() -> Factory[Module]:
    def adder(a: Port[UInt(32)], b: Port[UInt(32)]):
        a, b = pop_all(True)  # Pop all input ports
        c = a + b
        log("Adder: {} + {} = {}", a, b, c)
    return adder
```

Key characteristics:
- **Input Ports**: Receive data through typed input ports
- **Sequential Execution**: Operate with stage boundaries
- **Async Communication**: Make async calls to other modules
- **Credit Consumption**: Consume credits when executing

#### Downstream (Combinational Logic)

`Downstream` modules implement pure combinational logic that converges data from
multiple sources without stage boundaries:

```python
@factory(Downstream)
def adder_factory(a: Value, b: Value) -> Factory[Downstream]:
    def adder():
        a_val = a.optional(UInt(32)(1))
        b_val = b.optional(UInt(32)(1))
        c = a_val + b_val
        log("downstream: {} + {} = {}", a_val, b_val, c)
    return adder
```

Key characteristics:
- **No Input Ports**: Receive data through combinational pins
- **Combinational Execution**: Operate without stage boundaries
- **Data Convergence**: Combine data from multiple sources in the same cycle
- **Pin-based Communication**: Use exposed pins for data access

### Communication Patterns

The experimental frontend supports two primary communication patterns:

#### Sequential Communication (Module to Module)

Sequential communication occurs between pipeline stages using the credit system:

```python
@factory(Module)
def driver_factory(adder: Factory[Module]) -> Factory[Module]:
    def driver():
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        with if_(cnt[0] < UInt(32)(100)):
            (adder << (cnt[0], cnt[0]))()  # Bind and call
    return driver
```

The `<<` operator binds arguments to module ports, and `()` executes an async call.
This implements the credit-based communication where:
- The caller increases the callee's credit counter
- The callee consumes credits when executing
- Data flows through stage registers (FIFOs)

#### Combinational Communication (Module to Downstream)

Combinational communication enables same-cycle data processing:

```python
@factory(Module)
def forward_data_factory() -> Factory[Module]:
    def forward_data(data: Port[UInt(32)]):
        data = module.pop_all(True)
        pin(data)  # Expose as combinational pin
    return forward_data

@factory(Downstream)
def adder_factory(a: Value, b: Value) -> Factory[Downstream]:
    def adder():
        a_val = a.optional(UInt(32)(1))
        b_val = b.optional(UInt(32)(1))
        c = a_val + b_val
        log("downstream: {} + {} = {}", a_val, b_val, c)
    return adder

# Usage
lhs = forward_data_factory()
rhs = forward_data_factory()
driver_factory(lhs, rhs)
adder_factory(lhs.pins[0], rhs.pins[0])  # Combinational connection
```

The `pin()` function exposes internal values as combinational pins that can be
accessed by downstream modules without stage boundaries.

## Implementation Details

### Factory Decorator

The `@factory` decorator implements a sophisticated construction pipeline:

1. **Argument Validation**: Validates factory function arguments against type annotations
2. **Type-specific Validation**: Calls module-specific validation methods
3. **Module Creation**: Instantiates the module using type-specific constructors
4. **Naming**: Assigns unique names through the naming manager
5. **AST Construction**: Executes the inner function within module context
6. **Factory Wrapping**: Returns the module wrapped in a `Factory` instance

### Factory Wrapper

The `Factory` class provides a unified interface for different module types:

```python
class Factory(Generic[ModuleLike]):
    module: ModuleLike  # Underlying module instance
    pins: Optional[list[Value]]  # Exposed combinational pins
    
    def expose(self, *pins: Value) -> 'Factory[ModuleLike]':
        """Expose combinational pins for external access"""
        
    def __class_getitem__(cls, item: Type[Any]) -> Type['Factory']:
        """Create specialized factory subclasses"""
```

### ModuleFactory Extensions

`ModuleFactory` provides syntactic sugar for module communication:

```python
class ModuleFactory(Factory[Module]):
    def __lshift__(self, args):
        """Bind arguments using << operator"""
        
    def __call__(self):
        """Execute async call using () operator"""
```

The operator overloads enable intuitive syntax:
- `(module << args)()` for binding and calling
- `(module << {'port': value})()` for keyword binding
- `(module << (val1, val2))()` for positional binding

## Comparison with Legacy Frontend

### Legacy Frontend (Imperative)

The legacy frontend uses class-based construction:

```python
class Driver(Module):
    def __init__(self):
        super().__init__(ports={})
    
    @module.combinational
    def build(self):
        cnt = RegArray(UInt(32), 1)
        (cnt & self)[0] <= cnt[0] + UInt(32)(1)
        log('cnt: {}', cnt[0])
```

Characteristics:
- **Class-based**: Modules are Python classes
- **Imperative**: Explicit construction and method calls
- **Manual Context**: Developers manage module contexts manually
- **Direct Access**: Direct access to module attributes

### Experimental Frontend (Functional)

The experimental frontend uses factory functions:

```python
@factory(Module)
def driver_factory() -> Factory[Module]:
    def driver():
        cnt = RegArray(UInt(32), 1)
        cnt[0] = cnt[0] + UInt(32)(1)
        log('cnt: {}', cnt[0])
    return driver
```

Characteristics:
- **Function-based**: Modules are created through factory functions
- **Declarative**: Construction is handled by the decorator
- **Automatic Context**: Module contexts are managed automatically
- **Type-safe**: Type annotations provide compile-time checking

## Benefits

### Improved Composability

The functional approach enables better module composition:

```python
# Easy composition of modules
adder = adder_factory()
multiplier = multiplier_factory()
driver = driver_factory(adder, multiplier)
```

### Type Safety

Type annotations provide compile-time validation:

```python
@factory(Module)
def processor_factory(alu: Factory[Module]) -> Factory[Module]:
    def processor(data: Port[UInt(32)]):  # Type-checked port
        # Implementation
    return processor
```

### Reduced Boilerplate

The factory pattern eliminates repetitive construction code:

```python
# Automatic naming, context management, and AST construction
@factory(Module)
def my_module_factory() -> Factory[Module]:
    def my_module():
        # Just the logic, no boilerplate
    return my_module
```

### Better Error Messages

Type-specific validation provides clearer error messages:

```python
# Clear error if port types don't match
@factory(Module)
def bad_module_factory() -> Factory[Module]:
    def bad_module(wrong_type: int):  # TypeError: must be Port[DataType]
        pass
    return bad_module
```

## Future Directions

### Callback Module Type

The experimental frontend design includes provisions for a `Callback` module type,
though this is currently marked as RFC (Request for Comments). Callbacks would
enable event-driven programming patterns within the credit-based pipeline.

### Enhanced Type System

Future versions may include more sophisticated type checking, including:
- Generic module types
- Dependent types for port relationships
- Compile-time verification of communication patterns

### Performance Optimizations

The factory pattern enables several optimization opportunities:
- Lazy module construction
- Compile-time module specialization
- Dead code elimination based on usage patterns

## Conclusion

The experimental functional frontend represents a significant architectural
improvement over the legacy frontend, providing better composability, type safety,
and developer experience while maintaining compatibility with Assassyn's
credit-based pipeline architecture. The factory pattern enables functional
programming patterns that make module construction more intuitive and less
error-prone, while the unified interface ensures consistency across different
module types.

The design successfully bridges the gap between high-level functional programming
patterns and low-level hardware generation, making Assassyn more accessible to
developers familiar with functional programming while preserving the performance
and correctness guarantees of the underlying credit-based pipeline system.
