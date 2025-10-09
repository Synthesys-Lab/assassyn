# Module and Port Definitions

This file provides the core Abstract Syntax Tree (AST) implementations for hardware modules, their ports, and connection wires.

-----

## Exposed Interfaces

```python
class Module(ModuleBase): ...
class Port: ...
class Wire: ...

# Decorator for defining a Module's logic
@combinational
def my_module_logic(self, ...): ...
```

-----

## Module Class

The `Module` class is the primary AST node for defining a hardware module, inheriting from `ModuleBase`.

  * **Initialization**: A module is created with a dictionary of its `Port`s. It automatically registers itself with the system builder upon instantiation.
  * **Timing Policies**: Modules can operate with different timing policies, such as `Timing.SYSTOLIC` or `Timing.BACKPRESSURE`, which can be set implicitly via helper methods.
  * **Helper Methods**: It provides syntactic sugar like `pop_all_ports` to consume all inputs at once and `validate_all_ports` to wait until all inputs are valid.
  * **Module Calls**: The `bind()` and `async_called()` methods are frontend APIs for creating `Bind` and `AsyncCall` operations to connect to and call the module.

-----

## Port Class

The `Port` class defines a typed communication port for a `Module`, which behaves like a FIFO queue.

  * **Typed Creation**: Ports are created with a specific data type (`DType`). A convenient `Port[Bits(32)]` syntax is supported for instantiation.
  * **FIFO Operations**: It provides frontend APIs to generate IR for standard FIFO operations:
      * `valid()`: Checks if the port's FIFO has data.
      * `peek()`: Reads data without consuming it.
      * `pop()`: Consumes and returns data.
      * `push(value)`: Pushes data into the port's FIFO.

-----

## Wire Class

The `Wire` class represents a simple connection point, often used for external module interfaces.

  * It is defined with a data type (`DType`) and an optional `direction` ('input' or 'output').
  * The `assign(value)` method is used to connect a value to the wire.

-----

## @combinational Decorator

This decorator is applied to the function that defines a `Module`'s logic. It is created by the `combinational_for` factory and provides essential features like automatic IR context management and signal naming based on the Python source code.
