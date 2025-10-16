# External SystemVerilog Modules

`external.py` defines the frontend surface for integrating black-box SystemVerilog blocks into an Assassyn design. It layers a small amount of metadata and wiring helpers on top of `Downstream` so external IP behaves like a native module during IR construction, Verilog generation, and simulation.

-----

## Exposed Interfaces

```python
class ExternalSV(Downstream): ...

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

ip = MyIP()
ip.in_assign(a=value_a, b=value_b)
result = ip.y  # or ip.y[0] for RegOut
```

-----

## Descriptor Helpers

  * `WireIn[...]`, `WireOut[...]`, `RegOut[...]` are wrapper classes used in class annotations. They encode the port direction and wire kind in their type identity (WireIn=input wire, WireOut=output wire, RegOut=output reg) and wrap the element type (`DType`).
  * `Input`/`Output` remain as deprecated aliases for backward compatibility.

-----

## @external Decorator

  * The decorator validates that the class extends `ExternalSV`, walks `__annotations__`, and gathers all `WireIn`/`WireOut`/`RegOut` definitions into `_ExternalConfig`.
  * Configuration fields such as `__source__`, `__module_name__`, `__has_clock__`, `__has_reset__`, and `__no_arbiter__` are captured so users can override them either on the subclass or at instantiation time.
  * Getter/setter properties are installed lazily for each annotated attribute, enabling attribute-style access (`ext.y`) to read or drive wires.

-----

## Directional Wire Views

  * `DirectionalWires` exposes a dict-like API for inputs and outputs (`module.in_wires`, `module.out_wires`).
  * Reads defer to `_ensure_output_exposed`, which enters a builder context when necessary to create a `wire_read` expression; registered outputs return an `_ExternalRegOutProxy` that enforces index `0`.
  * Writes dispatch to `wire_assign` and update the underlying `Wire` so that subsequent IR traversal sees the same connection. Assignments can happen through `module.in_wires[...]`, direct attribute access, bracket syntax (`module['a'] = ...`), or the convenience wrapper `in_assign(...)`.

-----

## ExternalSV Class

  * **Construction**: resolves decorator supplied defaults, requires a `file_path`, and records `external_module_name`, `has_clock`, `has_reset`, along with module attributes like `Module.ATTR_EXTERNAL`.
  * **Wire Registration**: instantiates real `Wire` objects for every declared input/output, storing them in `self._wires`. Optional keyword arguments passed to the constructor are validated and queued until a builder context is available (`_apply_pending_connections`).
  * **IR Integration**: `in_assign()` pushes the builder onto the module body, drives any provided inputs, and returns the declared outputs in order (single object or tuple). Output reads are memoized per wire to keep the generated IR minimal.
  * **Indexing Helpers**: `__getitem__` and `__setitem__` forward to the directional adapters, letting users treat the module like a small associative array of ports.
  * **String Dump**: `__repr__` renders the external metadata, attached attributes, and the module body so debug dumps clearly mark external instantiations.

-----

## Registered Outputs

  * `_ExternalRegOutProxy` provides a read-only wrapper that mimics `RegArray` indexing semantics. It only accepts index `0`, returning the associated `wire_read` expression, and exposes the output `dtype` for convenience in type-sensitive code.

-----

## Typical Usage Patterns

  * **Combinational module**: `python/ci-tests/test_easy_external.py` uses `WireOut` to publish an external adder result directly to downstream logic.
  * **Pipelined handshake**: `python/ci-tests/test_pipemul.py` declares `RegOut` ports for `out_valid` and the product register while coordinating `async_called` updates.
  * **Nested instantiation**: `python/ci-tests/test_complex_external.py` constructs multiple `ExternalSV` instances inside a regular `Module`, chaining combinational logic with stateful registers.

These helpers ensure external IP participates naturally in the Assassyn IR while leaving the code generation and simulation stages to handle the concrete SystemVerilog glue.
