# Unified Factory Decorator

This module provides the `@factory(type)` decorator for constructing different types
of modules in the experimental frontend, including `Module`, `Downstream`, and `Callback`.
It implements a unified factory pattern that normalizes module construction across
different module flavors while providing type-specific validation and construction.

## Design Documents

- [Experimental Frontend Design](../../../docs/design/lang/experimental_fe.md) - Experimental frontend architecture
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Module Factory Support](./module.md) - Module factory support
- [Downstream Factory Support](./downstream.md) - Downstream module factory support
- [Naming Manager](../../builder/naming_manager.md) - Naming system
- [Builder Singleton](../../builder/__init__.md) - Builder context management

## Summary

The factory decorator provides a higher-order function approach to module construction
in Assassyn's experimental frontend. It enables a functional programming style where
modules are constructed through factory functions that return inner module definitions.
This approach provides better composability and type safety compared to the legacy
frontend's imperative construction patterns.

The decorator handles shared plumbing across all module types while delegating
type-specific validation and construction to specialized factory methods implemented
by each module type.

**Factory Pattern Singleton Usage:** The factory system integrates with the builder singleton for context management:
- **Context Entry**: Factory functions enter the builder context to manage module construction
- **AST Construction**: The inner function executes within the module context for proper AST building
- **Context Exit**: Factory functions clean up the builder context after module construction
- **Singleton Integration**: The factory pattern relies on the global builder singleton for context management

This singleton pattern enables the factory decorator to manage module construction contexts while maintaining the functional programming style of the experimental frontend.

## Exposed Interfaces

### factory

```python
def factory(module_type: Any) -> Callable[[Callable[..., Callable[..., Any]]], Callable[..., Factory[Any]]]:
    """Universal factory decorator."""
```

**Purpose**: Decorates factory functions to create modules of the specified type.

**Parameters**:
- `module_type`: The type of module to create (e.g., `Module`, `Downstream`)

**Returns**: A decorator function that wraps factory functions

**Explanation**: The decorator performs several key operations:
1. **Argument Validation**: Validates that arguments passed to the outer factory function
   match their type annotations using `_validate_outer_arguments()`
2. **Type-specific Validation**: Calls `module_type.factory_check_signature(inner)` to
   validate the inner function signature according to module type requirements
3. **Module Creation**: Calls `module_type.factory_create(inner, args)` to instantiate
   the module
4. **Naming**: Assigns a unique, capitalized name using the naming manager
5. **AST Construction**: Executes the inner function within the module context to build
   the module's AST
6. **Factory Wrapping**: Returns the module wrapped in a `Factory[module_type]` instance

The `module_type` must provide `factory_check_signature` and `factory_create` static methods
for type-specific validation and construction.

### Factory

```python
class Factory(Generic[ModuleLike]):
    """Generic wrapper returned by `@factory` decorated functions."""
```

**Purpose**: Generic wrapper class that provides a unified interface for different
module types returned by factory functions.

**Attributes**:
- `module`: The underlying module instance produced by the decorator
- `pins`: Optional list of combinational pins exposed via `expose()`

**Methods**:

#### __init__

```python
def __init__(self, module: ModuleLike):
    """Initialize the factory wrapper."""
```

**Purpose**: Creates a factory wrapper around a module instance.

**Parameters**:
- `module`: The module instance to wrap

#### expose

```python
def expose(self, *pins: Value) -> 'Factory[ModuleLike]':
    """Expose combinational pins to upstream modules."""
```

**Purpose**: Exposes combinational pins from the module for use by upstream modules.

**Parameters**:
- `*pins`: Variable number of `Value` objects to expose as pins

**Returns**: Self for method chaining

**Explanation**: This method is typically called at the end of module construction
to expose internal values as combinational pins that can be accessed by other
modules without stage boundaries.

#### __class_getitem__

```python
def __class_getitem__(cls, item: Type[Any]) -> Type['Factory']:
    """Return (and cache) a specialised Factory subclass for `item`."""
```

**Purpose**: Creates specialized factory subclasses for different module types.

**Parameters**:
- `item`: The module type to specialize for

**Returns**: A specialized `Factory` subclass

**Explanation**: This method enables type-safe factory creation using `Factory[ModuleType]`
syntax. It creates and caches specialized subclasses for each module type.

### this

```python
def this():
    """Return the module currently being constructed."""
```

**Purpose**: Returns the module currently being constructed within the factory context.

**Returns**: The current module instance from the singleton builder

**Explanation**: This function provides access to the current module being constructed,
enabling the inner function to reference its own module instance during construction.

### pin

```python
def pin(*pins: Value) -> None:
    """Expose combinational pins from the current module being constructed."""
```

**Purpose**: Exposes combinational pins from the current module being constructed.

**Parameters**:
- `*pins`: Variable number of `Value` objects to expose as pins

**Raises**: `RuntimeError` if called outside an active module context

**Explanation**: This function adds pins to the current module's pin list, making
them available for combinational connections to other modules. It must be called
within an active module construction context.

## Internal Helpers

### _validate_outer_arguments

```python
def _validate_outer_arguments(func: Callable[..., Any], args: tuple, kwargs: dict) -> Dict[str, Any]:
    """Validate arguments passed to the outer factory function."""
```

**Purpose**: Validates that arguments passed to factory functions match their type annotations.

**Parameters**:
- `func`: The factory function being called
- `args`: Positional arguments
- `kwargs`: Keyword arguments

**Returns**: Dictionary of validated arguments

**Explanation**: This function performs runtime type checking on factory function arguments
by comparing actual argument types against function annotations. It handles `Union` types
(including `Optional`) and falls back to trusting the caller for complex annotations.

### _verify_inner_name

```python
def _verify_inner_name(outer_name: str, inner_name: str) -> None:
    """Ensure the inner function follows the `<name>[_factory]` convention."""
```

**Purpose**: Validates that inner function names follow the expected naming convention.

**Parameters**:
- `outer_name`: Name of the outer factory function
- `inner_name`: Name of the inner function

**Raises**: `ValueError` if naming convention is violated

**Explanation**: Ensures inner functions are named consistently with their factory functions,
removing the `_factory` suffix if present. This maintains naming consistency across
the experimental frontend.

### _rename_module

```python
def _rename_module(module: Any, inner_name: str) -> None:
    """Assign a unique, capitalised module name derived from `inner_name`."""
```

**Purpose**: Assigns a unique, capitalized name to the module instance.

**Parameters**:
- `module`: The module instance to rename
- `inner_name`: The inner function name to derive the module name from

**Explanation**: Uses the naming manager to generate a unique, capitalized module name
based on the inner function name, ensuring no naming conflicts across module instances.

### _enter_module_context

```python
def _enter_module_context(module: Any) -> Block:
    """Initialise a module body and enter the builder context."""
```

**Purpose**: Initializes the module's body and enters the builder context for AST construction.

**Parameters**:
- `module`: The module instance to initialize

**Returns**: The module's body block

**Explanation**: Creates a new `Block` for the module body and enters the builder context
to enable AST construction within the module.

### _exit_module_context

```python
def _exit_module_context() -> None:
    """Exit the current module context."""
```

**Purpose**: Exits the current module construction context.

**Explanation**: Cleans up the builder context after module construction is complete.
