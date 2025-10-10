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


  - `send_read_request(mem, addr)`: It calls the corresponding `mi_<mem>` memory interface's `send_request` method with the given `address` and associated `callback_of_<mem>` discussed in [modules.md](../modules.md), and `is_write` set to false.
  - `read_request_succ(mem)`: It reads the value from `sim.<mem>_response.read_succ` as declared in [simulator.md](../simulator.md).
  - `send_write_request(mem, addr, data)`: It calls the corresponding `mi_<mem>` memory interface's `send_request` method with the given `address` and associated `callback_of_<mem>` discussed in [modules.md](../modules.md), and `is_write` set to true.
  - `write_request_succ(mem)`: It reads the value from `sim.<mem>_response.write_succ` as declared in [simulator.md](../simulator.md).
  - `has_mem_resp(mem)`: It checks if `sim.<mem>_response.valid`.
  - `get_mem_resp(mem)`: Get the memory response data. The lsb are the data payload, and the msb are the corresponding request address.
