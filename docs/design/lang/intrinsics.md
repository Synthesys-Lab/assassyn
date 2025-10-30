# Intrinsic Functions

Intrinsic functions are built-in operations that provide hardware-specific functionality not expressible through regular expressions.
They are essential for controlling execution flow, memory operations, and synchronization in Assassyn designs.

For the broader context of how intrinsics fit into the DSL, see [dsl.md](./dsl.md).
For implementation details, see [trace.md](./trace.md).

## Categories

Intrinsics are divided into two categories:
- **`Intrinsic`**: Operations with side effects that may vary based on inputs and system state
- **`PureIntrinsic`**: Pure operations that always produce the same output for the same inputs

---

## Execution Control Intrinsics

### `push_condition(condition)` / `pop_condition()`

Purpose: Manage a predicate stack that guards subsequent operations.

Parameters:
- `condition: Value` (for `push_condition`) – condition to push onto the predicate stack

Returns: `Intrinsic` – Non-valued operations that update the predicate stack

Usage:
```python
@module.combinational
def build(self):
    cond = self.counter[0] < UInt(32)(100)
    push_condition(cond)
    # Statements below are guarded by cond
    log("in range: {}", self.counter[0])
    pop_condition()
```

Nesting:
```python
@module.combinational
def build(self):
    c1 = self.counter[0] < UInt(32)(10)
    c2 = (self.counter[0] & UInt(32)(1)) == UInt(32)(0)
    push_condition(c1)
    push_condition(c2)
    # Guarded by c1 & c2
    log("even under 10: {}", self.counter[0])
    pop_condition()
    pop_condition()
```

Simulator Codegen: `push_condition` emits an `if (cond) {` and increases indentation; `pop_condition` closes the block `}`.

Verilog Codegen: The dumper maintains a predicate stack used by `get_pred()` gating; push/pop only update this stack and do not emit direct code.

Per-Module Semantics: The predicate stack is managed by the builder per module via a ModuleContext record. Each module has its own predicate stack; conditions do not leak across modules. Both explicit predicate intrinsics and `with Condition(cond): ...` push/pop the same builder-managed predicate stack. `Condition` is a thin sugar around `push_condition(cond)` / `pop_condition()`.

### `get_pred()`

Purpose: Return the current predicate computed as the AND of all active conditions on the predicate stack.

Parameters:
- None

Returns: `Value` – `Bits(1)` predicate; if the stack is empty, returns constant `1`.

Source of Truth: `get_pred()` computes the AND over the current module's predicate stack from the builder's ModuleContext. If invoked outside any module context, it returns `Bits(1)(1)`.

Usage:
```python
@module.combinational
def build(self):
    # Guard some enable with current predicate
    enable = get_pred()
    assume(enable)  # example usage
```

### `log(fmt, *values)`

Purpose: Emit a formatted trace message guarded by the current predicate.

Relationship to Predicates: The frontend helper captures the builder’s current predicate carry when the `Log` node is created. The base `Expr` stores this value in `meta_cond`, giving every backend a single guard without threading extra operands or replaying the predicate stack.

Verilog Codegen: The Verilog dumper exposes only the predicate referenced by `meta_cond` once, generating a `valid_*` / `expose_*` pair that drives the trace guard. Older logic walked the entire condition stack to expose each guard separately; the carry capture now serves as the single source of truth while keeping the IR lightweight.

Simulator Parity: The Python simulator follows the same contract—`meta_cond` controls when the `print` executes—so the behaviour stays consistent across backends.

### Predicate Metadata on Array/FIFO/Async Operations

Purpose: Provide a unified predicate contract for side-effecting operations beyond `log()`, enabling backends to skip redundant condition-stack reconstruction.

Relationship to Predicates: `Expr` hoists predicate capture for all nodes, so valued and side-effect operations automatically acquire `meta_cond` at construction time. Existing helpers (e.g., `ArrayWrite`, `FIFOPush`, `FIFOPop`, `AsyncCall`, and valued intrinsics) simply forward explicit overrides when they need to deviate from the ambient predicate stack.

Backend Consumption:
- **Verilog** uses `expr.meta_cond` when recording FIFO interactions and array writes, matching simulator gating without formatting the predicate stack.
- **Simulator** threads the same metadata into the generated Rust glue so runtime events only materialize when the predicate is true.

Legacy Compatibility: Older nodes that predate the metadata snapshot should migrate to the new helpers; current constructors always populate `meta_cond` (final carry). When no guard is present, `meta_cond` resolves to `Bits(1)(1)`.

### `wait_until(condition)`

**Purpose**: Block execution until a condition becomes true.

**Parameters**:
- `condition: Value` - The condition to wait for

**Returns**: `Intrinsic` - The wait-until intrinsic node

**Usage**:

**With `pop_all_ports(True)` - implicit wait_until**:
```python
@module.combinational
def build(self):
    a, b = self.pop_all_ports(True)  # Implicitly waits until all ports are valid
    result = a + b
```

**With `pop_all_ports(False)` - explicit wait_until**:
```python
@module.combinational
def build(self):
    wait_until(a.valid() & b.valid())  # First wait until both are valid
    a, b = self.pop_all_ports(False)  # Then unconditionally pop ports
    result = a + b
```

**Direct wait_until usage**:
```python
@module.combinational
def build(self, lock, sqr: Squarer):
    wait_until(lock[0])  # Wait until lock[0] is true
    a = self.pop_all_ports(False)
    sqr.async_called(a = a)
```

**Credit System Integration**: The `wait_until` intrinsic is the mechanism by which modules **consume credits**. When a module executes `wait_until`, it decreases its credit counter, indicating that it has consumed one credit for this activation.

**Important Notes**:
- All expressions before `wait_until` are executed regardless of success
- All expressions after `wait_until` are only executed when the condition is met
- This intrinsic is commonly used in CPU decoders to wait for operand validity

### `finish()`

**Purpose**: Terminate the simulation.

**Returns**: `Intrinsic` - The finish intrinsic node

**Usage**:
```python
@module.combinational
def build(self):
    with Condition(self.counter[0] >= UInt(32)(1000)):
        finish()  # Stop simulation
```

**Use Cases**:
- Stop simulation after a certain number of cycles
- Terminate when a specific condition is met
- End testbench execution

### `assume(condition)`

**Purpose**: Assert that a condition is true (renamed to avoid Python keyword conflict).

**Parameters**:
- `condition: Value` - The condition to assert

**Returns**: `Intrinsic` - The assert intrinsic node

**Usage**:
```python
@module.combinational
def build(self):
    result = self.adder(a, b)
    assume(result < UInt(32)(1000))  # Assert result is within bounds
```

**Use Cases**:
- Debugging and formal verification
- Design constraints validation
- Testbench assertions

---

## Memory Intrinsics

### `send_read_request(memory, read_enable, address)`

**Purpose**: Send a read request to memory.

**Parameters**:
- `memory: Value` - The memory module
- `read_enable: Value` - Read enable signal
- `address: Value` - Address to read from

**Returns**: `Intrinsic` - Boolean indicating if request was sent

**Usage**:
```python
@module.combinational
def build(self):
    send_read_request(self.dram, self.read_enable, self.addr)
```

### `send_write_request(memory, write_enable, address, data)`

**Purpose**: Send a write request to memory.

**Parameters**:
- `memory: Value` - The memory module
- `write_enable: Value` - Write enable signal
- `address: Value` - Address to write to
- `data: Value` - Data to write

**Returns**: `Intrinsic` - Boolean indicating if request was sent

**Usage**:
```python
@module.combinational
def build(self):
    send_write_request(self.dram, self.write_enable, self.addr, self.data)
```

### `has_mem_resp(memory)`

**Purpose**: Check if memory has a pending response.

**Parameters**:
- `memory: Value` - The memory module to check

**Returns**: `PureIntrinsic` - Boolean indicating response availability

**Usage**:
```python
@module.combinational
def build(self):
    with Condition(has_mem_resp(self.dram)):
        data = get_mem_resp(self.dram)
        self.process_data(data)
```

### `get_mem_resp(memory)`

**Purpose**: Get memory response data.

**Parameters**:
- `memory: Value` - The memory module

**Returns**: `PureIntrinsic` - Memory response data

**Usage**:
```python
@module.combinational
def build(self):
    with Condition(has_mem_resp(self.dram)):
        response = get_mem_resp(self.dram)
        # Extract data from response (LSB contains data, MSB contains address)
        data = response[0:31]  # Assuming 32-bit data
        addr = response[32:63]  # Assuming 32-bit address
```

**Response Format**: The response data format is handled by the code generation system. In the Python implementation, the data is returned as a `Value` object that can be used in expressions.

---

## System State Intrinsics

### `fifo_valid(fifo)`

**Purpose**: Check if FIFO has valid data.

**Parameters**:
- `fifo: Value` - The FIFO to check

**Returns**: `PureIntrinsic` - Boolean indicating FIFO validity

### `fifo_peek(fifo)`

**Purpose**: Peek at FIFO data without consuming.

**Parameters**:
- `fifo: Value` - The FIFO to peek

**Returns**: `PureIntrinsic` - FIFO data without consumption

### `module_triggered(module)`

**Purpose**: Check if a module was triggered this cycle.

**Parameters**:
- `module: Value` - The module to check

**Returns**: `PureIntrinsic` - Boolean indicating module activation

**Credit System Integration**: This intrinsic indicates whether a module **was triggered this cycle**, meaning its credit counter was decreased. This happens when the module executed a `wait_until` intrinsic, consuming one credit.

### `value_valid(value)`

**Purpose**: Check if a value is valid.

**Parameters**:
- `value: Value` - The value to check

**Returns**: `PureIntrinsic` - Boolean indicating value validity

---

### `current_cycle()`

**Purpose**: Get the current global cycle count for conditional scheduling and testbench logic.

**Parameters**:
- None

**Returns**: `PureIntrinsic` - `UInt(64)` current cycle number

**Usage**:
```python
@module.combinational
def build(self):
    from assassyn.ir.dtype import UInt
    from assassyn.ir.expr.intrinsic import current_cycle
    with Condition(current_cycle() == UInt(64)(10)):
        # Executes at cycle 10
        do_something()
```

**Notes**:
- `Cycle(n)` is now a thin wrapper around `Condition(current_cycle() == UInt(64)(n))`.
- Testbench scheduling in the simulator triggers the Testbench every cycle; guards can be applied using `current_cycle()`.

## Memory Request Patterns

### Basic Memory Access Pattern

```python
@module.combinational
def build(self):
    # Send read request
    send_read_request(self.dram, self.read_enable, self.addr)
    
    # Check for response
    with Condition(has_mem_resp(self.dram)):
        data = get_mem_resp(self.dram)
        self.process_data(data)
```

### Write Request Pattern

```python
@module.combinational
def build(self):
    # Send write request
    send_write_request(self.dram, self.write_enable, self.addr, self.data)
    
    # Note: Write requests don't return data, only success status
```

### Scope Limitations

Due to scope limitations in the trace-based DSL, memory request results cannot be directly used outside their conditional blocks:

```python
# This won't work as expected:
with Condition(we):
    x = send_write_request(self)  # x is only valid within this scope
# x is not accessible here
```

To handle this, use separate intrinsics to check request success:
- `has_mem_resp(mem)` - Check if memory has a response
- `get_mem_resp(mem)` - Get the memory response data
