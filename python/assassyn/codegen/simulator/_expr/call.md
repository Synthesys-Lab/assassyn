# Call Operation Code Generation

This module generates Rust code for call-related IR nodes, such as asynchronous calls and FIFO manipulations, for the simulator backend. These operations are fundamental to the inter-module communication in Assassyn's pipelined architecture, as described in the [simulator design document](../../../../docs/design/simulator.md).

## Summary

This module implements code generation for four types of call-related operations:
- **AsyncCall**: Schedules asynchronous module activation through event queues
- **FIFOPop**: Retrieves data from FIFO buffers with proper timing
- **FIFOPush**: Sends data to FIFO buffers with proper timing  
- **Bind**: No-op operation for simulation (used for binding parameters)

The timing model follows the simulator's half-cycle mechanism where pipeline stages execute at different time stamps (0, 25, 50, 100) within each cycle.
When semantic coverage is enabled, generated call operations also record source-level async-call and FIFO events.

## Exposed Interfaces

### codegen_async_call

```python
def codegen_async_call(node: AsyncCall, module_ctx) -> str
```

Schedules an asynchronous event by pushing a future timestamp onto the callee's event queue. The timestamp is calculated to trigger the callee module in the next cycle.

**Parameters:**
- `node`: The AsyncCall IR node containing the bind operation
- `module_ctx`: The current module context

**Returns:** Rust code string that schedules the async call

**Generated Code:**
```rust
{
    let stamp = sim.stamp - sim.stamp % 100 + 100;
    sim.<callee_name>_event.push_back(stamp);
    coverage.record_async_call(...);
}
```

**Explanation:**
The function calculates a timestamp for the next cycle (current cycle + 100) and pushes it to the callee's event queue. This follows the simulator's timing model where pipeline stages are triggered at cycle boundaries. The callee module checks its event queue and executes when the timestamp matches the current simulation time. The optional coverage probe uses source-level class names for the caller and callee.

### codegen_fifo_pop

```python
def codegen_fifo_pop(node: FIFOPop, module_ctx) -> str
```

Requests a value from a FIFO buffer. It logs a pop event and attempts to retrieve the front item, returning `false` if the FIFO is empty.

**Parameters:**
- `node`: The FIFOPop IR node containing the FIFO reference
- `module_ctx`: The current module context

**Returns:** Rust code string that pops from the FIFO

**Generated Code:**
```rust
{
    let stamp = sim.stamp - sim.stamp % 100 + 50;
    sim.<fifo_id>.pop.push(FIFOPop::new(stamp, "<module_name>"));
    coverage.record_fifo_pop(...);
    match sim.<fifo_id>.payload.front() {
        Some(value) => value.clone(),
        None => return false,
    }
}
```

**Explanation:**
The function schedules a pop operation at the half-cycle timestamp (current cycle + 50) and immediately attempts to retrieve the front value. If the FIFO is empty, the generated simulator reports the source location of the pop attempt. The optional coverage probe records the source-level FIFO ID before the value is read.

### codegen_fifo_push

```python
def codegen_fifo_push(node: FIFOPush, module_ctx) -> str
```

Adds a timestamped push request containing the value to the target FIFO's push queue.

**Parameters:**
- `node`: The FIFOPush IR node containing the FIFO reference and value
- `module_ctx`: The current module context

**Returns:** Rust code string that pushes to the FIFO

**Generated Code:**
```rust
{
    let stamp = sim.stamp;
    sim.<fifo_id>.push.push(
        FIFOPush::new(stamp + 50, <value>.clone(), "<module_name>"));
    coverage.record_fifo_push(...);
}
```

**Explanation:**
The function schedules a push operation at the half-cycle timestamp (current cycle + 50) with the value to be pushed. The value is cloned to ensure proper ownership in Rust. This implements the non-blocking behavior of FIFO push operations. The optional coverage probe records push counts and the configured depth when an explicit FIFO depth is attached to the push.

### codegen_bind

```python
def codegen_bind(node: Bind, module_ctx) -> str
```

Generates a no-op operation for simulation, returning the Rust unit type `()`.

**Parameters:**
- `node`: The Bind IR node (unused in simulation)
- `module_ctx`: The current module context (unused)

**Returns:** Rust code string `"()"`

**Explanation:**
Bind operations are used for parameter binding in the IR but have no runtime effect in simulation. The function simply returns the Rust unit type `()` to maintain type consistency.

## Internal Helpers

### `_coverage_fifo_id`

```python
def _coverage_fifo_id(fifo) -> str
```

Builds the source-level FIFO coverage ID `fifo:<module>.<port>`.

### `_coverage_module_name`

```python
def _coverage_module_name(module) -> str
```

Returns the source-level module class name used in coverage IDs.

### `_coverage_depth`

```python
def _coverage_depth(push: FIFOPush) -> int
```

Converts an explicit FIFO depth log2 annotation to a concrete depth for coverage replay.
