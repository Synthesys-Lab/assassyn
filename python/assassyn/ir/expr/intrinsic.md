# Intrinsic Functions

This module declares intrinsic operations and implements their frontend builders with `@ir_builder` annotation. Intrinsics are built-in operations that provide hardware-specific functionality not expressible through regular expressions.

Intrinsics are divided into two categories:
- **`Intrinsic`**: Operations with side effects that may vary based on inputs and system state
- **`PureIntrinsic`**: Pure operations that always produce the same output for the same inputs

## Design Documents

- [Intrinsic Operations Design](../../../docs/design/lang/intrinsics.md) - Intrinsic operations architecture and design
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Type System Design](../../../docs/design/lang/type.md) - Type system architecture and data type definitions
- [Memory System Architecture](../../../docs/design/arch/memory.md) - Memory system design including DRAM integration

## Related Modules

- [Expression Base](../expr.md) - Base expression classes and operand system
- [Arithmetic Operations](../arith.md) - Arithmetic and logical operations
- [Array Operations](../array.md) - Array read/write operations
- [Call Operations](../call.md) - Async call operations

---

## Exposed Interfaces

### Core Classes

#### `class Intrinsic(Expr)`

The class for intrinsic operations with side effects. These operations may have varying behavior based on system state and inputs.

**Constants:**
- `WAIT_UNTIL = 900` - Wait until a condition becomes true
- `FINISH = 901` - Terminate simulation
- `ASSERT = 902` - Assert a condition (renamed to `assume` to avoid Python keyword conflict)
- `SEND_READ_REQUEST = 906` - Send a read request to memory
- `SEND_WRITE_REQUEST = 908` - Send a write request to memory
- `EXTERNAL_INSTANTIATE = 913` - Instantiate and drive an external module (created implicitly by `ExternalSV` calls)

**Fields:**
- `opcode: int` - Operation code for this intrinsic

**Methods:**
- `__init__(opcode, *args, meta_cond=None)` - Initialize the intrinsic with opcode and arguments. The constructor forwards `meta_cond` to the base `Expr`, which records the current predicate carry (defaulting to [`get_pred()`](intrinsic.md#get_pred) when omitted).
- `args` - Get the arguments of this intrinsic (property)
- `dtype` - Get the data type of this intrinsic (property)
- `__enter__()` - Context manager entry
- `__exit__(exc_type, exc_value, traceback)` - Context manager exit

`Intrinsic` inherits `meta_cond` from `Expr`, so backends consume predicate metadata in a uniform way across all expression types.

#### `class PureIntrinsic(Expr)`

The class for pure intrinsic operations without side effects. These operations always produce deterministic results.

**Constants:**
- `FIFO_VALID = 300` - Check if FIFO has valid data
- `FIFO_PEEK = 303` - Peek at FIFO data without consuming
- `MODULE_TRIGGERED = 304` - Check if module is triggered
- `VALUE_VALID = 305` - Check if value is valid
- `EXTERNAL_OUTPUT_READ = 306` - Read an output port from an `ExternalIntrinsic`
- `HAS_MEM_RESP = 904` - Check if memory has response
- `GET_MEM_RESP = 912` - Get memory response data

**Methods:**
- `__init__(opcode, *args, meta_cond=None)` - Initialize the pure intrinsic with opcode and arguments, forwarding `meta_cond` to the base `Expr` so predicate carries are captured automatically (defaults to `get_pred()` when omitted).
- `args` - Get the arguments of this intrinsic (property)
- `dtype` - Get the data type of this intrinsic (property)

Pure intrinsics reuse the same predicate metadata accessor defined on `Expr`, making valued nodes participate in the same control-flow instrumentation as side-effect operations.

### Frontend Functions

#### `def wait_until(cond) -> Intrinsic`

Frontend API for creating a wait-until block.

**Parameters:**
- `cond: Value` - The condition to wait for

**Returns:**
- `Intrinsic` - The wait-until intrinsic node

**Explanation:**
This intrinsic blocks execution until the given condition becomes true. It's commonly used in pipeline stages to wait for valid data before proceeding. The condition is evaluated each cycle until it becomes true. 

**Credit System Integration:** The `wait_until` intrinsic is the mechanism by which modules **consume credits**. When a module executes `wait_until`, it decreases its credit counter, indicating that it has consumed one credit for this activation. The `driver` module has infinite credits and makes async calls that increase downstream module credits, while `wait_until` executions consume those credits.

For the complete design and architecture of the credit-based flow control system, see [pipeline.md](../../../docs/design/pipeline.md).

#### `def assume(cond) -> Intrinsic`

Frontend API for creating an assertion. This name avoids conflict with the Python keyword.

**Parameters:**
- `cond: Value` - The condition to assert

**Returns:**
- `Intrinsic` - The assert intrinsic node

**Explanation:**
This intrinsic asserts that a condition is true. If the condition is false during simulation, it will cause an assertion failure. This is useful for debugging and formal verification.

#### `def finish() -> Intrinsic`

Finish the simulation.

**Returns:**
- `Intrinsic` - The finish intrinsic node

**Explanation:**
This intrinsic terminates the simulation when executed. It's commonly used to stop simulation after a certain number of cycles or when a specific condition is met.

#### `def has_mem_resp(memory) -> PureIntrinsic`

Check if there is a memory response.

**Parameters:**
- `memory: Value` - The memory module to check

**Returns:**
- `PureIntrinsic` - The has_mem_resp intrinsic node

**Explanation:**
This pure intrinsic checks whether the specified memory module has a pending response. It returns a boolean value indicating response availability.

#### `def send_read_request(mem, re, addr) -> Intrinsic`

Send a read request with address to the given memory system.

**Parameters:**
- `mem: Value` - The memory module
- `re: Value` - Read enable signal
- `addr: Value` - Address to read from

**Returns:**
- `Intrinsic` - The send_read_request intrinsic node

**Explanation:**
This intrinsic sends a read request to the specified memory module. If the read enable signal is not asserted, no request is sent. Returns a boolean indicating if the request was successfully sent.

#### `def send_write_request(mem, we, addr, data) -> Intrinsic`

Send a write request with address and data to the given memory system.

**Parameters:**
- `mem: Value` - The memory module
- `we: Value` - Write enable signal
- `addr: Value` - Address to write to
- `data: Value` - Data to write

**Returns:**
- `Intrinsic` - The send_write_request intrinsic node

**Explanation:**
This intrinsic sends a write request to the specified memory module. If the write enable signal is not asserted, no request is sent. Returns a boolean indicating if the request was successfully sent.

#### `class ExternalIntrinsic(Intrinsic)`

Wrapper created automatically when calling an `ExternalSV` class. It records the external class, input connections, and exposes read-only accessors for output ports.

**Key Behaviours:**
- Input ports are passed positionally via keyword arguments, validated against the external class's `_wires` metadata.
- Output ports are accessed using attribute syntax (`instance.port`). Wire outputs return a `PureIntrinsic(EXTERNAL_OUTPUT_READ)` node; register outputs return an `_ExternalRegOutProxy` that enforces index 0 and generates the same intrinsic under the hood.
- The intrinsic's `uid` property is used by code generation to create stable handle names in both Verilog and the simulator.
- The intrinsic returns `Bits(1)` to integrate with existing expose/validity tracking but its logical payload is the external module instance.

#### `def external_instantiate(external_class, **inputs) -> ExternalIntrinsic`

Frontend helper invoked by the `ExternalSV` metaclass; regular users simply call `MyExternalSV(a=x, b=y)`.

**Explanation:**
This intrinsic materialises an external module instance, wiring all declared inputs. Subsequent attribute accesses on the returned object yield `PureIntrinsic(EXTERNAL_OUTPUT_READ)` nodes or register proxies.

#### `def get_mem_resp(mem) -> PureIntrinsic`

Get the memory response data.

**Parameters:**
- `mem: Value` - The memory module

**Returns:**
- `PureIntrinsic` - The get_mem_resp intrinsic node

**Explanation:**
This pure intrinsic retrieves the response data from the specified memory module. The least significant bits contain the data payload, while the most significant bits contain the corresponding request address. For generality, the response data is in `Vec<8>` format.

**Note on Memory Response Format:** The memory response data format is handled by the code generation system. In the Python implementation, the data is returned as a `Value` object that can be used in expressions. The actual data format conversion (e.g., from `Vec<8>` to `BigUint`) is handled during code generation to Rust.

#### `def external_output_read(instance, port_name, index=None) -> PureIntrinsic`

Thin wrapper used internally by `_ExternalRegOutProxy` and the attribute accessors on `ExternalIntrinsic`. Users normally obtain these by reading `instance.port` or `instance.port[idx]`.

**Explanation:**
Produces a pure intrinsic that references an external module output. When `index` is supplied, the intrinsic represents a register output access; otherwise it is a simple wire read. The dtype is derived from the external class metadata.

**Error Conditions:**
- Memory access errors: May occur if memory modules are not properly initialized
- Credit system errors: May occur if `wait_until` is called without proper credit allocation
- Assertion failures: May occur if conditions in `assume` intrinsics are false during simulation
- Memory response errors: May occur if memory response data is accessed before it's available

#### `MODULE_TRIGGERED` (PureIntrinsic)

Check if a module was triggered this cycle (credit was decreased).

**Credit System Integration:** This intrinsic indicates whether a module **was triggered this cycle**, meaning its credit counter was decreased. This happens when the module executed a `wait_until` intrinsic, consuming one credit. It's used internally by the credit-based flow control system to track module activation status.

For the complete design and architecture of the credit-based flow control system, see [pipeline.md](../../../docs/design/pipeline.md).

### Helper Functions

#### `def is_wait_until(expr) -> bool`

Check if the expression is a wait-until intrinsic.

**Parameters:**
- `expr: Expr` - The expression to check

**Returns:**
- `bool` - True if the expression is a wait-until intrinsic

---

## Internal Helpers

### Intrinsic Information Tables

The module defines two information tables that map opcodes to intrinsic metadata:

#### `INTRIN_INFO`
Maps intrinsic opcodes to tuples containing:
- `mnemonic: str` - Human-readable name
- `num_args: int` - Number of expected arguments
- `valued: bool` - Whether the intrinsic returns a value
- `side_effect: bool` - Whether the intrinsic has side effects

#### `PURE_INTRIN_INFO`
Maps pure intrinsic opcodes to tuples containing:
- `mnemonic: str` - Human-readable name
- `num_args: int` - Number of expected arguments

### DRAM Intrinsics

The DRAM intrinsics support per-DRAM-module memory interfaces with proper callback handling and response management, replacing the previous single global memory interface approach.

**Memory Request Pattern:**
```python
with Condition(we):
    x = send_write_request(self)
# We cannot get x here, as x is outside the scope.
```

To handle scope limitations, separate intrinsics are provided to check if memory requests were successful:
- `has_mem_resp(mem)` - Check if memory has a response
- `get_mem_resp(mem)` - Get the memory response data

### Data Type Handling

Memory response data is handled in `Vec<8>` format for generality. Conversion from `Vec<u8>` to integer uses `BigUint::from_bytes_le` followed by `ValueCastTo<>` to cast to the desired destination type.

### Operator Mapping

The `OPERATORS` dictionary maps pure intrinsic opcodes to their string representations for debugging and IR dumps.
