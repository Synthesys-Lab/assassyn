# PyCDE Wrapper Helpers

## Summary

`assassyn.pycde_wrapper` centralises PyCDE helper factories that the Verilog backend and hand-authored designs share. The module exposes reusable FIFO, trigger-counter, and register-file constructions that mirror the SystemVerilog resources shipped alongside the backend. Keeping these helpers in one place ensures generated designs and user code instantiate identical primitives without duplicating boilerplate.

## Exposed Interfaces

### `FIFO`

```python
@modparams
def FIFO(WIDTH: int, DEPTH_LOG2: int):
    """Depth-parameterized FIFO matching the backend's SystemVerilog resource."""
```

Creates a PyCDE `Module` compatible with `python/assassyn/codegen/verilog/fifo.sv`. The returned class exposes:

- Inputs: `clk`, active-low `rst_n`, `push_valid`, `push_data`, `pop_ready`
- Outputs: `push_ready`, `pop_valid`, `pop_data`

**Project-specific knowledge required**:
- Understanding of the handshake protocol described in [`docs/design/internal/pipeline.md`](../docs/design/internal/pipeline.md)
- Familiarity with the FIFO SystemVerilog implementation in [`python/assassyn/codegen/verilog/fifo.sv`](./codegen/verilog/fifo.sv)

### `TriggerCounter`

```python
@modparams
def TriggerCounter(WIDTH: int):
    """Credit counter primitive used to gate driver execution."""
```

Produces a PyCDE `Module` mirroring `python/assassyn/codegen/verilog/trigger_counter.sv`. It keeps the driver trigger credit pool in sync with async callers by matching the valid/ready handshake.

Ports:
- Inputs: `clk`, `rst_n`, `delta`, `pop_ready`
- Outputs: `delta_ready`, `pop_valid`

**Project-specific knowledge required**:
- Credit-based scheduling rules in [`docs/design/internal/pipeline.md`](../docs/design/internal/pipeline.md)

### `build_register_file`

```python
def build_register_file(
    module_name: str,
    data_type,
    depth: int,
    num_write_ports: int,
    num_read_ports: int,
    *,
    addr_width: int | None = None,
    include_read_index: bool = True,
    initializer: list[int] | None = None,
):
    """Create a parameterized register file module with the requested port counts."""
```

Constructs a multi-port register file `Module` used by the Verilog backend to materialise shared arrays. The helper mirrors the interface that `CIRCTDumper.visit_array` previously emitted inline:

- Always provides `clk: Clock` and `rst: Reset` ports.
- For each write port `i`, declares `w_port<i>: Input(Bits(1))`, `widx_port<i>: Input(Bits(addr_width))`, and `wdata_port<i>: Input(data_type)`.
- For each read port `i`, declares `rdata_port<i>: Output(data_type)` and, when `include_read_index` is `True`, `ridx_port<i>: Input(Bits(addr_width))`.

Runtime behaviour:
- Stores the array contents in a `Reg(dim(data_type, depth))` with reset value derived from `initializer` when provided, otherwise zeros. Literal entries are coerced into `data_type` so mixed signed/unsigned uses remain well-typed.
- Applies write-port updates in descending index order, matching the Verilog backend’s historical last-writer-wins priority.
- When read indices are present, performs a simple linear mux over the registered data to select each reader’s output.

Parameter notes:
- `addr_width` defaults to `max(1, ceil_log2(depth))` to keep port widths stable for single-entry arrays.
- Setting `include_read_index=False` omits `ridx_port<i>` inputs. The backend uses this for width-one arrays where indices are constant.
- `initializer` should contain `depth` entries that match the array element semantics (Python ints are acceptable; the helper casts them to the PyCDE type).

**Project-specific knowledge required**:
- Array ownership and metadata rules in [`python/assassyn/codegen/verilog/array.md`](./codegen/verilog/array.md)
- Background on register arrays in [`docs/design/internal/array-ownership.md`](../docs/design/internal/array-ownership.md)

## Internal Helpers

The module does not currently expose additional helpers; the three primary factories above encapsulate the supported runtime primitives. When adding new PyCDE wrappers, follow the same pattern—parameterise via `@modparams`, forward reset wiring exactly as the matching SystemVerilog resource expects, and document the interface here to keep runtime and generated designs aligned.
