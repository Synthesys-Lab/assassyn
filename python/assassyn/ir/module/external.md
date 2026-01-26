# External SystemVerilog Modules

`external.py` defines the frontend surface for integrating black-box SystemVerilog blocks into an Assassyn design. It layers a small amount of metadata and wiring helpers on top of `Downstream` so external IP behaves like a native module during IR construction, Verilog generation, and simulation.

-----

## Exposed Interfaces

```python
class ExternalSV: ...  # descriptor only; calling it yields an ExternalIntrinsic

class WireIn(Generic[DType]): ...
class WireOut(Generic[DType]): ...
class RegOut(Generic[DType]): ...

@external
class MyIP(ExternalSV):
    a: WireIn[UInt(32)]
    b: WireIn[UInt(32)]
    y: WireOut[UInt(32)]
    __source__ = "rtl/my_ip.sv"
    __module_name__ = "my_ip"
    __parameters__ = {"ADDR_WIDTH": 8, "INIT_FILE": "rtl/init.hex"}

inst = MyIP(a=value_a, b=value_b)  # returns ExternalIntrinsic
result = inst.y          # wire output → PureIntrinsic(EXTERNAL_OUTPUT_READ)
```

-----

## Descriptor Helpers

  * `WireIn[...]`, `WireOut[...]`, `RegOut[...]` are wrapper classes used in class annotations. They encode the port direction and wire kind in their type identity (WireIn=input wire, WireOut=output wire, RegOut=output reg) and wrap the element type (`DType`).
  * `Input`/`Output` remain as deprecated aliases for backward compatibility.

-----

## @external Decorator

  * The decorator validates that the class extends `ExternalSV`, walks `__annotations__`, and gathers all `WireIn`/`WireOut`/`RegOut` definitions into the `_wires` metadata table.
  * Configuration fields such as `__source__`, `__module_name__`, `__has_clock__`, and `__has_reset__` are captured so code generation stages can decide how to wrap and clock the external block.
  * `__parameters__` (optional dict) captures SystemVerilog parameter overrides; values may be `int`/`bool`/`str` (or `PathLike`, converted to `str`) and are surfaced to both Verilator (`-G`) and PyCDE wrapper generation.
  * The decorated class remains callable; invoking it runs through the metaclass and returns an `ExternalIntrinsic` instead of a Python object. There is no longer a mutable Python instance that exposes setters/getters.

-----

## ExternalSV Descriptor

  * **Construction**: The metaclass intercepts calls to the class and routes them to `_create_external_intrinsic`, which wraps the request in an `ExternalIntrinsic`.
  * **Metadata**: `_wires` stores port declarations (direction + kind + dtype) while `_metadata` records auxiliary fields such as `module_name`, `source`, clock/reset booleans, etc. Downstream code generation stages read these tables directly.
  * **Parameters**: `__parameters__` is folded into `_metadata["parameters"]` so simulator + Verilog backends can apply SV parameter overrides consistently.
  * **No Mutable Instance**: The descriptor no longer inherits from `Downstream` or exposes mutation APIs like `in_assign`. All connectivity is described by the returned intrinsic and its operands.
  * **Debugging Support**: `__repr__` is implemented on the Python side for better logging, but day-to-day interaction happens through the intrinsic nodes.

-----

## Registered Outputs

  * `_ExternalRegOutProxy` provides a read-only wrapper that mimics `RegArray` indexing semantics. It only accepts index `0`, returning the associated `PureIntrinsic(EXTERNAL_OUTPUT_READ)` expression, and exposes the output `dtype` for convenience in type-sensitive code.

-----

## Typical Usage Patterns

  * **Combinational module**: `python/ci-tests/test_easy_external.py` uses `WireOut` to publish an external adder result directly to downstream logic.
  * **Pipelined handshake**: `python/ci-tests/test_pipemul.py` declares `RegOut` ports for `out_valid` and the product register while coordinating `async_called` updates.
  * **Nested instantiation**: `python/ci-tests/test_complex_external.py` constructs multiple `ExternalSV` instances inside a regular `Module`, chaining combinational logic with stateful registers.

These helpers ensure external IP participates naturally in the Assassyn IR while leaving the code generation and simulation stages to handle the concrete SystemVerilog glue.
