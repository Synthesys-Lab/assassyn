# Intrinsic Functions

This module declares intrinsic operations and implements their frontend builders with `@ir_builder` annotation. Intrinsics are built-in operations that provide hardware-specific functionality not expressible through regular expressions.

Intrinsics are divided into two categories:
- **`Intrinsic`**: Operations with side effects that may vary based on inputs and system state
- **`PureIntrinsic`**: Pure operations that always produce the same output for the same inputs

---

## Exposed Interfaces

### Core Classes

#### `class Intrinsic(Expr)`

The class for intrinsic operations with side effects. These operations may have varying behavior based on system state and inputs.

**Constants:**
- `WAIT_UNTIL = 900` - Wait until a condition becomes true
- `FINISH = 901` - Terminate simulation
- `ASSERT = 902` - Assert a condition (renamed to `assume` to avoid Python keyword conflict)
- `BARRIER = 903` - Create a barrier in the execution flow
- `SEND_READ_REQUEST = 906` - Send a read request to memory
- `SEND_WRITE_REQUEST = 908` - Send a write request to memory

**Fields:**
- `opcode: int` - Operation code for this intrinsic

**Methods:**
- `__init__(opcode, *args)` - Initialize intrinsic with opcode and arguments
- `args` - Get the arguments of this intrinsic (property)
- `dtype` - Get the data type of this intrinsic (property)
- `__enter__()` - Context manager entry
- `__exit__(exc_type, exc_value, traceback)` - Context manager exit

#### `class PureIntrinsic(Expr)`

The class for pure intrinsic operations without side effects. These operations always produce deterministic results.

**Constants:**
- `FIFO_VALID = 300` - Check if FIFO has valid data
- `FIFO_PEEK = 303` - Peek at FIFO data without consuming
- `MODULE_TRIGGERED = 304` - Check if module is triggered
- `VALUE_VALID = 305` - Check if value is valid
- `HAS_MEM_RESP = 904` - Check if memory has response
- `GET_MEM_RESP = 912` - Get memory response data

**Methods:**
- `__init__(opcode, *args)` - Initialize pure intrinsic with opcode and arguments
- `args` - Get the arguments of this intrinsic (property)
- `dtype` - Get the data type of this intrinsic (property)

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

#### `def barrier(node) -> Intrinsic`

Barrier the current simulation state.

**Parameters:**
- `node: Value` - The node to use as barrier

**Returns:**
- `Intrinsic` - The barrier intrinsic node

**Explanation:**
This intrinsic creates a barrier in the execution flow, ensuring that all previous operations complete before proceeding. It's used for synchronization in complex designs.

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

#### `def get_mem_resp(mem) -> PureIntrinsic`

Get the memory response data.

**Parameters:**
- `mem: Value` - The memory module

**Returns:**
- `PureIntrinsic` - The get_mem_resp intrinsic node

**Explanation:**
This pure intrinsic retrieves the response data from the specified memory module. The least significant bits contain the data payload, while the most significant bits contain the corresponding request address. For generality, the response data is in `Vec<8>` format.

**Note on Memory Response Format:** The memory response data format is handled by the code generation system. In the Python implementation, the data is returned as a `Value` object that can be used in expressions. The actual data format conversion (e.g., from `Vec<8>` to `BigUint`) is handled during code generation to Rust.

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