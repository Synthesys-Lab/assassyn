# Intrinsic Code Generation

This module generates Rust code for two types of special operations: side-effect-free `PureIntrinsic` functions and side-effecting `Intrinsic` commands.

-----

## Exposed Interfaces

```python
def codegen_pure_intrinsic(node: PureIntrinsic, module_ctx, sys) -> str
def codegen_intrinsic(node: Intrinsic, module_ctx, sys, ...) -> str
```

-----

## Pure Intrinsics

`codegen_pure_intrinsic` generates code for read-only operations that inspect the simulator state.

  * **`FIFO_PEEK`**: Peeks the front value of a FIFO without removing it.
      * **Generated Code**: `sim.<fifo>.front().cloned()`
  * **`FIFO_VALID`**: Checks if a FIFO is not empty.
      * **Generated Code**: `!sim.<fifo>.is_empty()`
  * **`VALUE_VALID`**: Checks if a signal's value is valid (`Some`).
      * **Generated Code**: `sim.<value>_value.is_some()`
  * **`MODULE_TRIGGERED`**: Checks if a module was triggered in the current cycle.
      * **Generated Code**: `sim.<module>_triggered`

-----

## Side-Effecting Intrinsics

`codegen_intrinsic` generates code for commands that can change the simulator state or control flow.

  * **`WAIT_UNTIL`**: Pauses the current module's execution until a condition is true.
      * **Generated Code**: `if !<condition> { return false; }`
  * **`FINISH`**: Terminates the entire simulation.
      * **Generated Code**: `std::process::exit(0);`
  * **`ASSERT`**: Asserts a runtime condition, causing a panic if it's false.
      * **Generated Code**: `assert!(<condition>);`
  * **`BARRIER`**: A no-op in generated code, used as a hint for compilation.
  * **`SEND_READ_REQUEST` / `SEND_WRITE_REQUEST`**: Send a read or write request to the main memory interface.
  * **`USE_DRAM`**: A configuration command that links a specific FIFO to receive DRAM read responses.
  * **`HAS_MEM_RESP` / `MEM_RESP`**: Check for and retrieve a pending data response from the DRAM-linked FIFO.
  * **`MEM_WRITE`**: Performs an array write operation specifically for DRAM.
