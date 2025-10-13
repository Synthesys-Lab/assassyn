# Intrinsic Code Generation

This module generates Rust code for intrinsic operations in the simulator backend. Intrinsics are special operations that provide hardware-specific functionality not expressible through regular expressions, including execution control, memory operations, and system state inspection.

For the broader context of intrinsics in the DSL, see [intrinsics.md](../../../docs/design/lang/intrinsics.md). For the credit-based pipeline architecture, see [arch.md](../../../docs/design/arch/arch.md).

## Summary

This module handles code generation for two categories of intrinsic operations:

1. **PureIntrinsic**: Side-effect-free operations that inspect simulator state (FIFO operations, memory responses, module state)
2. **Intrinsic**: Side-effecting operations that control execution flow and memory operations

The module uses dispatch tables to map intrinsic opcodes to their corresponding code generation functions, translating high-level intrinsic operations into Rust code that interfaces with the simulator runtime.

---

## Exposed Interfaces

### `codegen_pure_intrinsic`

```python
def codegen_pure_intrinsic(node: PureIntrinsic, module_ctx, sys) -> str
```

Generates Rust code for pure intrinsic operations that inspect simulator state without side effects.

**Parameters:**
- `node: PureIntrinsic` - The pure intrinsic node to generate code for
- `module_ctx` - Module context containing naming and type information
- `sys` - System context for simulator state access

**Returns:**
- `str` - Generated Rust code string, or `None` if intrinsic is not supported

**Explanation:**
This function dispatches to the appropriate code generation function based on the intrinsic's opcode. Pure intrinsics are used for inspecting FIFO state, checking memory responses, and querying module activation status. The generated code accesses simulator state through the `sim` object without modifying it.

### `codegen_intrinsic`

```python
def codegen_intrinsic(node: Intrinsic, module_ctx, sys, **kwargs) -> str
```

Generates Rust code for side-effecting intrinsic operations that control execution flow and perform memory operations.

**Parameters:**
- `node: Intrinsic` - The intrinsic node to generate code for
- `module_ctx` - Module context containing naming and type information
- `sys` - System context for simulator state access
- `**kwargs` - Additional keyword arguments passed to code generation functions

**Returns:**
- `str` - Generated Rust code string, or `None` if intrinsic is not supported

**Explanation:**
This function dispatches to the appropriate code generation function based on the intrinsic's opcode. Side-effecting intrinsics include execution control (`wait_until`, `finish`, `assert`), memory operations (`send_read_request`, `send_write_request`), and synchronization primitives (`barrier`). The generated code may modify simulator state or control execution flow.

---

## Internal Helpers

### Dispatch Tables

#### `_PURE_INTRINSIC_DISPATCH`

```python
_PURE_INTRINSIC_DISPATCH = {
    PureIntrinsic.FIFO_PEEK: _codegen_fifo_peek,
    PureIntrinsic.FIFO_VALID: _codegen_fifo_valid,
    PureIntrinsic.VALUE_VALID: _codegen_value_valid,
    PureIntrinsic.MODULE_TRIGGERED: _codegen_module_triggered,
    PureIntrinsic.HAS_MEM_RESP: _codegen_has_mem_resp,
    PureIntrinsic.GET_MEM_RESP: _codegen_get_mem_resp,
}
```

Maps pure intrinsic opcodes to their corresponding code generation functions.

#### `_INTRINSIC_DISPATCH`

```python
_INTRINSIC_DISPATCH = {
    Intrinsic.WAIT_UNTIL: _codegen_wait_until,
    Intrinsic.FINISH: _codegen_finish,
    Intrinsic.ASSERT: _codegen_assert,
    Intrinsic.BARRIER: _codegen_barrier,
    Intrinsic.SEND_READ_REQUEST: _codegen_send_read_request,
    Intrinsic.SEND_WRITE_REQUEST: _codegen_send_write_request,
}
```

Maps side-effecting intrinsic opcodes to their corresponding code generation functions.

### FIFO Operations

#### `_codegen_fifo_peek`

```python
def _codegen_fifo_peek(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to peek at the front value of a FIFO without removing it.

**Generated Code:** `sim.<fifo>.front().cloned()`

#### `_codegen_fifo_valid`

```python
def _codegen_fifo_valid(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to check if a FIFO is not empty.

**Generated Code:** `!sim.<fifo>.is_empty()`

### System State Operations

#### `_codegen_value_valid`

```python
def _codegen_value_valid(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to check if a signal's value is valid (Some).

**Generated Code:** `sim.<value>_value.is_some()`

#### `_codegen_module_triggered`

```python
def _codegen_module_triggered(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to check if a module was triggered in the current cycle.

**Generated Code:** `sim.<module>_triggered`

### Memory Operations

#### `_codegen_has_mem_resp`

```python
def _codegen_has_mem_resp(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to check if memory has a pending response.

**Generated Code:** `sim.<dram_name>_response.valid`

#### `_codegen_get_mem_resp`

```python
def _codegen_get_mem_resp(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to get memory response data, converting Vec<u8> to BigUint.

**Generated Code:** `BigUint::from_bytes_le(&sim.<dram_name>_response.data)`

### Execution Control Operations

#### `_codegen_wait_until`

```python
def _codegen_wait_until(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to pause execution until a condition is true.

**Generated Code:** `if !<condition> { return false; }`

**Explanation:**
This implements the credit-based pipeline mechanism where modules consume credits by executing `wait_until`. When the condition is false, the function returns `false`, indicating the module should not be activated this cycle.

#### `_codegen_finish`

```python
def _codegen_finish(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to terminate the simulation.

**Generated Code:** `std::process::exit(0);`

#### `_codegen_assert`

```python
def _codegen_assert(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to assert a runtime condition.

**Generated Code:** `assert!(<condition>);`

#### `_codegen_barrier`

```python
def _codegen_barrier(node, module_ctx, sys, **_kwargs) -> str
```

Generates a no-op barrier operation.

**Generated Code:** `/* Barrier */`

### Memory Request Operations

#### `_codegen_send_read_request`

```python
def _codegen_send_read_request(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to send a read request to memory.

**Generated Code:**
```rust
if <read_enable> {
    unsafe {
        let mem_interface = &sim.mi_<dram_name>;
        let success = mem_interface.send_request(
            <addr> as i64,
            false,
            crate::modules::<dram_name>::callback_of_<dram_name>,
            sim as *const _ as *mut _,
        );
        if success {
            sim.request_stamp_map_table.insert(
                <addr> as i64,
                sim.stamp,
            );
        }
        success
    }
} else {
    false
}
```

**Explanation:**
This generates unsafe Rust code that interfaces with the Ramulator2 memory simulator. It sends a read request through the memory interface and tracks the request timestamp for response matching.

#### `_codegen_send_write_request`

```python
def _codegen_send_write_request(node, module_ctx, sys, **_kwargs) -> str
```

Generates code to send a write request to memory.

**Generated Code:**
```rust
if <write_enable> {
    unsafe {
        let mem_interface = &sim.mi_<dram_name>;
        let success = mem_interface.send_request(
            <addr> as i64,
            true,
            crate::modules::<dram_name>::callback_of_<dram_name>,
            sim as *const _ as *mut _,
        );
        success
    }
} else {
    false
}
```

**Explanation:**
Similar to read requests but with `is_write=true`. Write requests don't return data, only success status.