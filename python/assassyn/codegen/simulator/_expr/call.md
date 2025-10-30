# Call Operation Code Generation

This module generates Rust code for call-related IR nodes, such as asynchronous calls and FIFO manipulations, for the simulator backend. These operations are fundamental to the inter-module communication in Assassyn's pipelined architecture, as described in the [simulator design document](../../../../docs/design/simulator.md).

## Summary

This module implements code generation for four types of call-related operations:
- **AsyncCall**: Schedules asynchronous module activation through event queues
- **FIFOPop**: Retrieves data from FIFO buffers with proper timing
- **FIFOPush**: Sends data to FIFO buffers with proper timing  
- **Bind**: No-op operation for simulation (used for binding parameters)

The timing model follows the simulator's half-cycle mechanism where pipeline stages execute at different time stamps (0, 25, 50, 100) within each cycle.

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
    if <predicate> {
        let stamp = sim.stamp - sim.stamp % 100 + 100;
        sim.<callee_name>_event.push_back(stamp);
    }
}
```

**Explanation:**
The function calculates a timestamp for the next cycle (current cycle + 100) and, when the captured predicate is true, pushes it to the callee's event queue. This matches the simulator's timing model while ensuring side effects are suppressed whenever the enclosing condition is false, mirroring the Verilog backend’s gating.

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
    if <predicate> {
        let stamp = sim.stamp - sim.stamp % 100 + 50;
        sim.<fifo_id>.pop.push(FIFOPop::new(stamp, "<module_name>"));
    }
    match sim.<fifo_id>.payload.front() {
        Some(value) => value.clone(),
        None => return false,
    }
}
```

**Explanation:**
The function records a pop request at the half-cycle timestamp (current cycle + 50) only when the predicate metadata evaluates to true. Regardless of whether an event was queued, it attempts to peek the front value, returning `false` if the FIFO is empty. This maintains the blocking behaviour while avoiding spurious events under disabled predicates.

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
    if <predicate> {
        let stamp = sim.stamp;
        sim.<fifo_id>.push.push(
            FIFOPush::new(stamp + 50, <value>.clone(), "<module_name>"));
    }
}
```

**Explanation:**
The function schedules a push operation at the half-cycle timestamp (current cycle + 50) with the provided value, cloning it to satisfy Rust ownership. The predicate guard ensures only enabled pushes enqueue events, keeping the simulator aligned with Verilog predicate handling.

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
