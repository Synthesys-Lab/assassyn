# Call Expression Generation

This module provides Verilog code generation for call operations, including async calls, wire assignments, and wire reads, with special handling for external SystemVerilog modules.

## Summary

The call expression generation module handles the conversion of Assassyn call operations into Verilog code. It manages async calls for the credit-based pipeline architecture, wire operations for inter-module communication, and provides special support for external SystemVerilog modules.

## Exposed Interfaces

### `codegen_async_call`

```python
def codegen_async_call(dumper, expr: AsyncCall) -> Optional[str]:
    """Generate code for async call operations."""
```

**Explanation**

This function handles async call operations by registering them with the module's expose mechanism. Async calls are fundamental to the credit-based pipeline architecture described in [arch.md](/docs/design/arch/arch.md).

The function performs the following steps:

1. **Expose Registration**: Calls `dumper.expose('trigger', expr)` to register the async call operation
2. **Deferred Processing**: The actual trigger logic is generated later during the cleanup phase

The cleanup phase handles async call operations by:
- Summing all trigger predicates for each target module
- Generating trigger signals that increment the target module's credit counter
- Creating proper credit-based flow control signals

This deferred approach allows the cleanup phase to:
- Group multiple async calls to the same module
- Generate appropriate trigger signal multiplexing
- Handle credit counter management

**Project-specific Knowledge Required**:
- Understanding of [async call operations](/python/assassyn/ir/expr/call.md)
- Knowledge of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Understanding of [expose mechanism](/python/assassyn/codegen/verilog/design.md)
- Reference to [cleanup phase processing](/python/assassyn/codegen/verilog/cleanup.md)

### `codegen_bind`

```python
def codegen_bind(_dumper, _expr: Bind) -> Optional[str]:
    """Generate code for bind operations.

    Bind operations don't generate any code, they just represent bindings.
    """
```

**Explanation**

This function handles bind operations, which represent parameter bindings in function calls. Bind operations don't generate any Verilog code directly as they are purely structural elements that organize the call parameters.

The function is intentionally empty because:
- Bind operations are processed during the async call generation
- They don't represent actual hardware operations
- The binding information is used by the async call handler to generate proper parameter connections

**Project-specific Knowledge Required**:
- Understanding of [bind operations](/python/assassyn/ir/expr/call.md)
- Knowledge of [parameter binding in calls](/python/assassyn/ir/expr/call.md)

### `codegen_wire_assign`

```python
def codegen_wire_assign(dumper, expr: WireAssign) -> Optional[str]:
    """Generate code for wire assign operations."""
```

**Explanation**

This function handles wire assignment operations, with special handling for external SystemVerilog modules. It performs the following steps:

1. **External Module Detection**: Checks if the wire assignment is to an external SystemVerilog module
2. **Input Registration**: For external modules, registers the wire assignment as a pending external input
3. **Comment Generation**: Returns a comment documenting the external wire assignment

The function handles external modules by:
- Identifying wires that belong to `ExternalSV` modules
- Storing wire assignments in `dumper.pending_external_inputs` for later processing
- Ensuring proper instantiation order for external modules

This approach allows external SystemVerilog modules to be properly instantiated with their required input connections.

**Project-specific Knowledge Required**:
- Understanding of [wire assignment operations](/python/assassyn/ir/expr/call.md)
- Knowledge of [external SystemVerilog modules](/python/assassyn/ir/module/external.md)
- Understanding of [wire operations](/python/assassyn/ir/module/base.md)

### `codegen_wire_read`

```python
def codegen_wire_read(dumper, expr: WireRead) -> Optional[str]:
    """Generate code for wire read operations."""
```

**Explanation**

This function generates Verilog code for wire read operations, with comprehensive handling for external SystemVerilog modules. It performs the following steps:

1. **Comment Generation**: Adds a comment documenting the external wire read
2. **External Module Instantiation**: Handles instantiation of external SystemVerilog modules
3. **Wire Access**: Generates code to read from the external module's output

The function handles external modules by:
- **Module Instantiation**: Creates instances of external modules with proper connections
- **Clock/Reset Handling**: Automatically connects clock and reset signals if required
- **Input Connection**: Connects all pending external inputs to the module instance
- **Output Access**: Generates code to read from the module's output wires

The instantiation process includes:
- Clock connection (`clk=self.clk`) if the module has a clock
- Reset connection (`rst=self.rst`) if the module has a reset
- Input connections from pending external inputs
- Proper instance naming and module reference

**Project-specific Knowledge Required**:
- Understanding of [wire read operations](/python/assassyn/ir/expr/call.md)
- Knowledge of [external SystemVerilog modules](/python/assassyn/ir/module/external.md)
- Understanding of [module instantiation](/python/assassyn/codegen/verilog/module.md)
- Reference to [right-hand value generation](/python/assassyn/codegen/verilog/rval.md)

## Internal Helpers

The module uses several utility functions:

- `dump_rval()` from [rval module](/python/assassyn/codegen/verilog/rval.md) for generating signal references
- `namify()` from [utils module](/python/assassyn/utils.md) for name generation

The call expression generation is integrated into the main expression dispatch system through the [__init__.py](/python/assassyn/codegen/verilog/_expr/__init__.md) module, which routes different expression types to their appropriate code generation functions.

**Project-specific Knowledge Required**:
- Understanding of [expression dispatch system](/python/assassyn/codegen/verilog/_expr/__init__.md)
- Knowledge of [CIRCTDumper integration](/python/assassyn/codegen/verilog/design.md)
- Reference to [call expression types](/python/assassyn/ir/expr/call.md)
- Understanding of [external module integration](/python/assassyn/ir/module/external.md)
