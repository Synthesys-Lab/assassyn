# Factory

This module provides `@factory(type)` decorator to construct
different types of modules, including `Module`, `Downstream`, and `Callback`.

## Exposed Interface

````python
@factory(<type>)
def my_module_factory(...) -> Factory[<type>]:
    def my_module(...):
        ...
    return my_module
````

Before calling the decorated function, the `@factory` decorator
first enforces all the arguments fed to the decorated function
are the same types as annotated, by iterating over the arguments and checking
their types in `__annotations__`, and then:

1. It extracts all the arguments in the returned inner function signature to
   perform type-specific checks by invoking `type.factory_check_signature(inner)`.
   - Refer to `module.md`, `downstream.md`, and `callback.md` for more details.
2. Iit creates the corresponding module object by invoking
   `type.factory_create(...)` with the extracted arguments.
   - Also refer to `module.md`, `downstream.md`, and `callback.md` for more details.
3. The module name will be renamed to the inner function name capitalized.
   - Check the inner function's name are the same as the outer factory function's name
     with the `_factory` suffix removed.
   - Because `Driver` module is reserved in the old frontend to be the top module,
     so a factory function named `driver_factory` should create a module named `Driver`.
     For now, we make this compromise to keep compatibility.
   - To avoid name collision, it uses `Singleton.naming_manager.get_module_name(inner.__name__)`.
     The inner function name is given to `get_module_name` to generate a unique name
     by increasing a counter of instantiations of each function.
4. Call the inner function to grow the AST of the module, and wrap the returned module
   in `Factory[...]` class (see below).
   - To grow the AST, refer to `@combinational_for` decorator implemented in
     [base.py](../../ir/module/base.py) to enter and exit the AST growing context.

## Factory Class

The `Factory[...]` class is a generic wrapper of a module
to construct `@factory`-decorated modules.

````python
class Factory:
    module: Module | Downstream | Callback # The current underlying module
    pins: list[Value] # Combinational pins exposed to external modules

    def __init__(self, module: Module | Downstream | Callback):
        self.module = module
        self.pins = None

    def expose(self, *pins: Value)
        '''Expose combinational pins to external modules.
        This method is typically called at the end of constructing a module.
        '''
    
    def __cls_getitem__(cls, item: Module | Downstream | Callback) -> type:
        '''A type annotation method to wrap different types of modules.
        This method is called when using `Factory[<type>]` annotation,
        which returns <type>Factory class, a subclass of Factory class.
        '''
````

## Singleton Wrapper

````python
def this(): 
    '''Returns the current module being constructed from `Singleton`.'''
````
