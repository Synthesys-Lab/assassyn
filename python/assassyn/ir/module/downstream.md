# Downstream 

This `Downstream` module type is used for creating logic that is combinational across multiple standard modules.

-----

## Exposed Interfaces

```python
class Downstream(ModuleBase):
    ...

# Decorator for defining the module's logic
@combinational
def my_downstream_logic(self, ...):
    ...
```

-----

## Downstream Class

The `Downstream` class is a special module container for logic that depends on outputs from several different chronological modules.

  * It inherits all the core functionality of `ModuleBase`, such as external dependency tracking.
  * When a `Downstream` module is instantiated, it automatically registers itself with the system builder, distinguishing it from regular modules.
  * Its string representation in the IR dump is explicitly marked with a `#[downstream]` attribute.

-----

## @combinational Decorator

This decorator is used on the function that defines the `Downstream` module's logic.

  * It is a specialized instance of the powerful `@combinational_for` decorator from the `base` module.
  * It provides all the same advanced features, including automatic IR context management and signal naming inferred from the Python source code.
