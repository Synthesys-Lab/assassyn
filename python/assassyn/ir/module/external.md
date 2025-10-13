# External SystemVerilog Module Integration

## Summary

The `ExternalSV` class enables integration of pre-existing SystemVerilog modules into Assassyn's credit-based pipeline architecture. This module provides a bridge between Assassyn's high-level IR and external SystemVerilog implementations, supporting both simulation and Verilog generation backends as described in the [pipeline design](../../../docs/design/internal/pipeline.md).

## Exposed Interfaces

### ExternalSV Class

```python
class ExternalSV(Module):
    def __init__(self, file_path, in_wires=None, out_wires=None, module_name=None, 
                 no_arbiter=False, has_clock=False, has_reset=False, **wire_connections): ...
    @property
    def wires(self): ...
    def __setitem__(self, key, value): ...
    def __getitem__(self, key): ...
    def in_assign(self, **kwargs): ...
    def __repr__(self): ...
```

### DirectionalWires Class

```python
class DirectionalWires:
    def __init__(self, ext_module, direction): ...
    def __contains__(self, key): ...
    def __iter__(self): ...
    def __getitem__(self, key): ...
    def __setitem__(self, key, value): ...
    def keys(self): ...
```

## Internal Helpers

### ExternalSV Class

The `ExternalSV` class extends the base `Module` class to integrate external SystemVerilog modules into Assassyn's IR.

**Purpose:** Provides a seamless interface for incorporating pre-existing SystemVerilog modules into Assassyn designs, handling wire declarations, assignments, and code generation.

**Member Fields:**
- `file_path: str` - Path to the SystemVerilog source file
- `external_module_name: str` - Name of the module in the SystemVerilog file
- `has_clock: bool` - Whether the external module requires clock signal
- `has_reset: bool` - Whether the external module requires reset signal
- `_wires: dict` - Dictionary of declared wires keyed by name
- `in_wires: DirectionalWires` - Adapter for input wire access
- `out_wires: DirectionalWires` - Adapter for output wire access

**Methods:**

#### `__init__(self, file_path, in_wires=None, out_wires=None, module_name=None, no_arbiter=False, has_clock=False, has_reset=False, **wire_connections)`

**Explanation:**
Constructs an external SystemVerilog module integration. The constructor:

1. **File Path Handling:** Stores the SystemVerilog file path, preserving relative paths for later resolution during elaboration
2. **Module Identification:** Sets the external module name (defaults to class name if not specified)
3. **Clock/Reset Configuration:** Records whether the external module requires clock and reset signals
4. **Wire Registration:** Creates `Wire` objects for declared input and output wires, registering them with the module's port definitions
5. **Directional Adapters:** Creates `DirectionalWires` adapters for convenient input/output access
6. **External Marking:** Tags the module with `Module.ATTR_EXTERNAL` for special handling in code generation
7. **Initial Connections:** Processes keyword arguments as initial wire assignments for declared input wires

The method ensures that all declared wires are properly typed and accessible through both the adapter interfaces and direct module access.

#### `wires` property

**Explanation:**
Exposes the internal wire dictionary for use by helper adapters and code generation passes. Returns the `_wires` dictionary keyed by wire name.

#### `__setitem__(self, key, value)`

**Explanation:**
Allows assignment to wires using bracket notation. Delegates to the appropriate directional adapter based on wire direction. Raises `KeyError` if the wire is not found.

#### `__getitem__(self, key)`

**Explanation:**
Allows access to wires using bracket notation. Checks output wires first, then input wires. Raises `KeyError` if the wire is not found.

#### `in_assign(self, **kwargs)`

**Explanation:**
Convenience method for assigning values to multiple input wires using keyword arguments. The method:
1. Assigns each keyword argument to the corresponding input wire
2. Collects all output wires
3. Returns the single output if only one exists, or a tuple of all outputs

This method is commonly used for connecting external modules to the rest of the design.

#### `__repr__(self)`

**Explanation:**
Generates the string representation for IR dumps. The method:
1. Formats port definitions
2. Includes module attributes
3. Adds external file information (file path and module name)
4. Generates the module declaration with external-specific formatting

The output follows Assassyn's IR format with additional external module metadata.

### DirectionalWires Class

The `DirectionalWires` class provides a convenient adapter for accessing wires based on their direction.

**Purpose:** Simplifies wire access by providing direction-specific interfaces that handle the complexity of wire assignment and reading operations.

**Member Fields:**
- `_module: ExternalSV` - Reference to the owning external module
- `_direction: str` - The wire direction this adapter handles ('input' or 'output')

**Methods:**

#### `__init__(self, ext_module, direction)`

**Explanation:**
Initializes the directional adapter with a reference to the external module and the specific direction it handles.

#### `_get_wire(self, key)`

**Explanation:**
Internal helper that retrieves a wire by name and validates its direction. Raises `KeyError` if the wire doesn't exist or `ValueError` if the wire's direction doesn't match the adapter's direction.

#### `__contains__(self, key)`

**Explanation:**
Checks if a wire exists and matches the adapter's direction. Returns `True` if the wire exists and has the correct direction.

#### `__iter__(self)`

**Explanation:**
Returns an iterator over all wire names that match the adapter's direction.

#### `__getitem__(self, key)`

**Explanation:**
Retrieves a wire value. For output wires, returns a `WireRead` expression. For input wires, returns the assigned value directly.

#### `__setitem__(self, key, value)`

**Explanation:**
Assigns a value to an input wire. Creates a `WireAssign` expression and updates the wire's assigned value. Raises `ValueError` if attempting to assign to an output wire.

#### `keys(self)`

**Explanation:**
Returns a list of all wire names that match the adapter's direction. Used for iteration and introspection.
