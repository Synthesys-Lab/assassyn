# Call Operation Code Generation

This module generates Rust code for call-related IR nodes, such as asynchronous calls and FIFO manipulations, for the simulator backend.

-----

## Exposed Interfaces

```python
def codegen_async_call(node: AsyncCall, module_ctx, sys) -> str
def codegen_fifo_pop(node: FIFOPop, module_ctx, sys, module_name) -> str
def codegen_fifo_push(node: FIFOPush, module_ctx, sys, module_name) -> str
def codegen_bind(node: Bind, module_ctx, sys) -> str
```

-----

## Async Call Generation

`codegen_async_call` schedules an asynchronous event by pushing a future timestamp onto the callee's event queue.

  * **Generated Code**:
    ```rust
    {
        let stamp = sim.stamp - sim.stamp % 100 + 100;
        sim.<callee_name>_event.push_back(stamp)
    }
    ```

-----

## FIFO Pop Generation

`codegen_fifo_pop` requests a value from a FIFO. It logs a pop event and tries to retrieve the item, returning `false` if the FIFO is empty.

  * **Generated Code**:
    ```rust
    {
        let stamp = sim.stamp - sim.stamp % 100 + 50;
        sim.<fifo_id>.pop.push(FIFOPop::new(stamp, "<module_name>"));
        match sim.<fifo_id>.payload.front() {
          Some(value) => value.clone(),
          None => return false,
        }
    }
    ```

-----

## FIFO Push Generation

`codegen_fifo_push` adds a timestamped push request, containing the value, to the target FIFO's `push` queue.

  * **Generated Code**:
    ```rust
    {
        let stamp = sim.stamp;
        sim.<fifo_id>.push.push(
          FIFOPush::new(stamp + 50, <value>.clone(), "<module_name>"));
    }
    ```

-----

## Bind Operation Generation

`codegen_bind` is a no-op for simulation, returning the Rust unit type `()`.
