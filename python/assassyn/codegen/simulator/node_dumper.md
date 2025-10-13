# Node Reference Dumper

This module provides functionality for converting Assassyn IR nodes into Rust code references during simulator generation. It implements a dispatch-based system that handles different types of nodes (arrays, ports, constants, modules, expressions) and generates appropriate Rust code for accessing their values in the simulator context.

## Section 0. Summary

The node dumper is a critical component of the simulator code generation pipeline. It translates Assassyn IR nodes into Rust code that can access values in the simulator context. The module uses a dispatch table pattern to handle different node types efficiently, with special handling for cross-module references, FIFO operations, and value cloning requirements.

## Section 1. Exposed Interfaces

### dump_rval_ref

```python
def dump_rval_ref(module_ctx, _, node):
    """Dispatch to appropriate handler based on node kind."""
```

**Explanation:**

This function serves as the main entry point for converting Assassyn IR nodes into Rust code references. It uses a dispatch table to route different node types to their appropriate handlers. The function first attempts an exact type match, then falls back to isinstance checks for subclasses, ensuring that all node types are handled appropriately.

The function handles the core logic of determining how to reference a node in the generated Rust code, taking into account the module context to determine whether a value is local or needs to be accessed through the simulator's exposed value mechanism.

## Section 2. Internal Helpers

### _handle_array

```python
def _handle_array(unwrapped, _module_ctx):
    """Handle Array nodes."""
    return namify(unwrapped.name)
```

**Explanation:**

Handles array references by converting the array name to a Rust identifier. Arrays in the simulator are accessed directly by name through the simulator context, so this handler simply returns the nameified array name.

### _handle_port

```python
def _handle_port(unwrapped, _module_ctx):
    """Handle Port nodes."""
    return fifo_name(unwrapped)
```

**Explanation:**

Handles port references by generating the appropriate FIFO name. Ports in the simulator are implemented as FIFOs, so this handler uses the `fifo_name` utility function to generate the correct FIFO reference name.

### _handle_const

```python
def _handle_const(unwrapped, _module_ctx):
    """Handle Const nodes."""
    return int_imm_dumper_impl(unwrapped.dtype, unwrapped.value)
```

**Explanation:**

Handles constant values by converting them to appropriate Rust literals. This handler delegates to the `int_imm_dumper_impl` function to handle the conversion from Assassyn data types to Rust types, ensuring proper type representation for immediate values.

### _handle_module

```python
def _handle_module(unwrapped, _module_ctx):
    """Handle Module nodes."""
    return namify(unwrapped.as_operand())
```

**Explanation:**

Handles module references by converting the module name to a Rust identifier. Modules are referenced by name in the simulator context, so this handler simply returns the nameified module name.

### _handle_expr

```python
def _handle_expr(unwrapped, module_ctx):
    """Handle Expr nodes."""
    # Figure out the ID format based on context
    parent_block = unwrapped.parent
    if module_ctx != parent_block.module:
        raw = namify(unwrapped.as_operand())
        field_id = f"{raw}_value"
        panic_log = f"Value {raw} invalid!"
        # Return as a block expression that evaluates to the value
        return f"""{{
                if let Some(x) = &sim.{field_id} {{
                    x
                }} else {{
                    panic!("{panic_log}");
                }}
            }}.clone()"""

    ref = namify(unwrapped.as_operand())
    if isinstance(unwrapped, PureIntrinsic) and unwrapped.opcode == PureIntrinsic.FIFO_PEEK:
        return f"{ref}.clone().unwrap()"

    dtype = unwrapped.dtype
    if dtype.bits <= 64:
        # Simple value
        return namify(ref)

    # Large value needs cloning
    return f"{ref}.clone()"
```

**Explanation:**

This is the most complex handler, dealing with expression nodes that can represent various types of values. The handler implements several important behaviors:

1. **Cross-module references**: When an expression belongs to a different module than the current context, it generates code to access the value through the simulator's exposed value mechanism. This involves checking if the value exists and panicking if it doesn't.

2. **FIFO peek operations**: Special handling for FIFO_PEEK intrinsics, which need to unwrap the optional value from the FIFO front.

3. **Value cloning**: For large values (>64 bits), the handler generates code to clone the value to avoid ownership issues in Rust.

4. **Simple references**: For small values, the handler generates a simple reference without cloning.

The handler demonstrates the complexity of managing value references across the simulator's module boundaries and the need for careful handling of Rust's ownership system.

### _handle_str

```python
def _handle_str(unwrapped, _module_ctx):
    """Handle string nodes."""
    return f'"{unwrapped}"'
```

**Explanation:**

Handles string literals by wrapping them in Rust string quotes. This is a simple handler that converts Python strings to Rust string literals.

### _RVAL_HANDLER_DISPATCH

```python
_RVAL_HANDLER_DISPATCH = {
    Array: _handle_array,
    Port: _handle_port,
    Const: _handle_const,
    Module: _handle_module,
    Expr: _handle_expr,
    str: _handle_str,
}
```

**Explanation:**

This dispatch table maps node types to their corresponding handler functions. The table is used by `dump_rval_ref` to route different node types to the appropriate handler. The dispatch pattern allows for efficient handling of different node types without complex if-else chains, making the code more maintainable and extensible.

The table includes handlers for all the major node types that can appear in Assassyn IR: arrays, ports, constants, modules, expressions, and strings. This comprehensive coverage ensures that all possible node types are handled appropriately during simulator code generation.
