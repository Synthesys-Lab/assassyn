# Downstream Module

## Summary

The `Downstream` module type implements combinational logic that operates across multiple chronological modules in Assassyn's credit-based pipeline architecture. Unlike regular modules that execute sequentially with credit-based flow control, downstream modules provide immediate, same-cycle communication between pipeline stages as described in the [architectural design](../../../docs/design/arch/arch.md).

## Exposed Interfaces

### Downstream Class

```python
class Downstream(ModuleBase):
    def __init__(self): ...
    @property
    def name(self) -> str: ...
    @name.setter
    def name(self, value: str): ...
    def _repr_impl(self, head): ...
    def __repr__(self): ...
```

### Combinational Decorator

```python
@combinational
def my_downstream_logic(self, ...):
    """
    Define combinational logic for the downstream module.
    
    @param self The downstream module instance
    @param ... Additional parameters as needed
    """
    ...
```

## Internal Helpers

### Downstream Class

The `Downstream` class is a specialized module container for combinational logic that depends on outputs from multiple chronological modules.

**Purpose:** Downstream modules enable cross-stage combinational communication in Assassyn's pipeline architecture, allowing immediate data flow between stages without credit-based delays.

**Member Fields:**
- `_name: str` - Internal storage for the module name
- `body: list[Expr]` - Ordered list of expressions representing the module's logic

**Methods:**

#### `__init__(self)`

**Explanation:**
Initializes a new downstream module instance. The constructor:
1. Calls the parent `ModuleBase.__init__()` to initialize base functionality
2. Determines the module name using the naming manager if available, or generates a default name
3. Initializes the body to `None` (will be set to a plain list by the `@combinational` decorator)
4. Registers the module with the system builder's downstream list for special handling during code generation

The naming follows Assassyn's naming conventions, with special handling for reserved names and automatic semantic name assignment.

#### `name` property

**Explanation:**
Provides access to the module's name for IR generation. When the name is explicitly set, it stores the name in the internal `_name` field for consistent access.

#### `_repr_impl(self, head)`

**Explanation:**
Generates the string representation for IR dumps. This method:
1. Sets the representation indentation level
2. Generates the module's operand identifier
3. Uses the shared `render_module_body()` helper to include the module body representation if available, so downstream dumps stay consistent with regular module dumps
4. Dumps external dependencies
5. Formats the output with the specified head attribute (typically "downstream")

The output follows Assassyn's IR format with external dependencies, module declaration, and body content.

#### `__repr__(self)`

**Explanation:**
Public interface for string representation, delegates to `_repr_impl` with "downstream" as the head attribute.

### Combinational Decorator

The `@combinational` decorator is a specialized instance of `combinational_for(Downstream)` from [base.py](base.md).

**Explanation:**
This decorator provides essential functionality for downstream module logic definition:

1. **IR Context Management:** Automatically enters and exits the module and body contexts in the IR builder
2. **Signal Naming:** Infers signal names from Python source code parameters
3. **AST Transformation:** Uses `rewrite_assign` to handle assignment operations
4. **Body Assignment:** Creates and assigns a list object to store the module's logic

The decorator ensures that all operations within the decorated function are properly recorded in the IR and that the module's body is correctly structured for downstream code generation stages.
