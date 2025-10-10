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

  - `send_read_request(mem, re, addr)`: if `re` is true, it calls
  `mi_<mem>.send_request(addr=address, is_write=false, callback=callback_of_<mem>)` as
  discussed in [modules.md](../modules.md), which returns
  if this read request is successful. If `re` is false, just give a `false`.
  - `send_write_request(mem, we, addr, data)`: Similar as above, it calls
  `mi_<mem>.send_request(addr=address, is_write=true, callback=callback_of_<mem>)`,
  which returns if this write request is successfully sent. If `we` is false, just give a `false`.
  - `has_mem_resp(mem)`: It checks if `sim.<mem>_response.valid`.
  - `get_mem_resp(mem)`: Get the memory response data. The lsb are the data payload, and the msb are the corresponding request address.
    - As Ramulator2 only simulates the memory behavior without holding any data, the data should be retrieved from the associated `_payload` array from the corresponding `DRAM`.