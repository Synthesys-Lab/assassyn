# SystemVerilog Generator

This backend lowers Assassyn IR into a PyCDE design (`design.py`), compiles it to SystemVerilog, and generates a cocotb/Verilator testbench (`tb.py`). It wires modules via FIFOs, surfaces cross‑module values, and materializes memory payloads (detected via `array.owner`) as simple blackboxes.

## Entry Points

```python
def elaborate(sys: SysBuilder, **kwargs) -> Path
```

- `sys`: The system to elaborate.
- `kwargs`: See Configuration. Common keys: `path`, `verilog`, `sim_threshold`, `resource_base`.
- Returns: Verilog output directory. When invoked via `assassyn.backend.elaborate`, this is `<path>/<sys.name>/verilog`.

Helper functions used by `elaborate`:

```python
def generate_design(fname: Union[str, Path], sys: SysBuilder) -> list[str]
def generate_top_harness(dumper) -> None
def generate_testbench(fname: Union[str, Path], sys: SysBuilder, sim_threshold: int,
                       dump_logger: list[str], external_files: list[str]) -> None
def generate_sram_blackbox_files(sys, path, resource_base=None) -> None
```

## What Gets Generated

- `design.py`: PyCDE design for all modules and the `Top` harness; calls `System([Top], name="Top", output_directory="sv").compile()`.
- `sv/`: Compiled SystemVerilog (e.g., `sv/hw/Top.sv`, `filelist.f`).
- `tb.py`: Cocotb testbench harness (Verilator runner).
- `fifo.sv`, `trigger_counter.sv`: Required SV resources.
- `sram_blackbox_<array>.sv`: One blackbox per SRAM payload array.
- Any `ExternalSV.file_path` sources referenced by the IR.

## High‑Level Architecture

- `CIRCTDumper` (design.py): IR visitor that emits PyCDE classes and the `Top` harness.
- Expression lowering (`_expr`): Lowers IR ops and records cross‑module “exposures”.
- Port synthesis (module.py): Declares module IO based on role (driver/downstream/SRAM) and usage.
- Cleanup (cleanup.py): Produces `executed_wire`, `finish`, FIFO push/pop, array write muxes, SRAM controls, and `expose_*`/`valid_*`.
- System assembly (system.py, top.py): Analyzes async callers, arrays, and memory payload ownership alongside externals; generates the `Top` netlist.
- Elaboration (elaborate.py): Writes `design.py`, compiles SV, emits blackboxes, copies resources, generates the testbench.

## PyCDE Header

The top of `design.py` imports shared parameterized wrappers used by `Top`:

```python
from pycde import Input, Output, Module, System, Clock, Reset, dim
from pycde import generator, modparams
from pycde.constructs import Reg, Array, Mux, Wire
from pycde.types import Bits, SInt, UInt
from assassyn.pycde_wrapper import FIFO, TriggerCounter, build_register_file
```

`assassyn.pycde_wrapper` centralizes PyCDE helpers that back the credit-based pipeline. It exposes:

- `FIFO`: Parameterized depth-tracking FIFO that maps to `fifo.sv`
- `TriggerCounter`: Credit counter primitive that maps to `trigger_counter.sv`
- `build_register_file`: Factory that produces multi-port register files matching the Verilog backend’s expectations (write-enable/index/data triplets plus optional read indices)

Keeping these definitions in a runtime module ensures generated designs and user-authored helpers reuse the same implementations.

## Design Content

`design.py` contains:

- Array writers: one class per array that stores the backing state, consumes write ports, and serves per-reader interfaces (`ridx_port<i>` / `rdata_port<i>`).
- External wrappers: one class per `ExternalSV` with `module_name` and declared IO.
- Module classes: one class per IR module. Common ports:
  - `clk: Clock`, `rst: Reset`, `cycle_count: Input(UInt(64))`
  - `executed: Output(Bits(1))`, `finish: Output(Bits(1))`
  - Driver‑only: `trigger_counter_pop_valid: Input(Bits(1))`
  - Per input port `<p>`: `<p>: Input(<ty>)`, `<p>_valid: Input(Bits(1))`, and if popped, `<p>_pop_ready: Output(Bits(1))`
  - Downstream externals: `<producer>_<value>: Input(<ty>)`, `<producer>_<value>_valid: Input(Bits(1))`
  - SRAM downstreams: `mem_address`, `mem_write_data`, `mem_write_enable`, `mem_read_enable`, `mem_dataout`
  - Arrays: readers drive `<a>_ridx_port<i>` and consume `<a>_rdata_port<i>`; writers drive `<a>_w_port<i>`, `<a>_wdata_port<i>`, `<a>_widx_port<i>`

### CIRCTDumper Walkthrough

- `visit_system` builds: `array_metadata`, external wrappers, SRAM metadata, and the cross-module exposure tables required for external wiring. Async-call and downstream dependencies are consumed from the frozen metadata and analysis helpers during module and top-level emission.
- `visit_module` walks the body (via `_expr`), declares ports (`generate_module_ports`, which infers roles and metadata internally), then emits handshakes and gating (`cleanup_post_generation`) inside `construct`.
- `visit_block` tracks nested predicates for conditional and cycled blocks so `Log`/`FINISH`/FIFO ops inherit the correct guards.

### Expression Lowering

- Arrays/FIFOs: `ArrayRead` produces a dedicated `rdata_port<i>` access (or `mem_dataout` inside SRAM), `ArrayWrite` marks exposure, `FIFOPop` reads `self.<p>`, `FIFOPush` exposes callee push intent.
- Intrinsics: `WAIT_UNTIL` contributes to `executed_wire` (drivers), `FINISH` contributes to `finish`, `VALUE_VALID`/`FIFO_PEEK`/`FIFO_VALID` produce signals or use `expose_*` when crossing modules, `Log` appends cocotb prints.
- Calls: `AsyncCall` exposes `<callee>_trigger` so callers can increment the callee’s trigger counter.

### RValue Naming

- Stable names for constants/ports/modules; expressions get unique `tmp`‑style names.
- Cross‑module references within downstreams are rewritten as `self.<producer>_<name>` so `Top` can connect `expose_*`/`valid_*` from producers.

## Handshake & Scheduling

- `executed_wire` gates side‑effects each cycle (built through `_format_reduction_expr` so OR / AND reductions share the same formatting):
  - Drivers: `trigger_counter_pop_valid [& WAIT_UNTIL]`
  - Downstreams: OR of upstream `inst_<dep>.executed`
- FIFO push (producer of `<C>.<p>`):
  - `<C>_<p>_push_valid = executed_wire & predicate & fifo_<C>_<p>_push_ready`
  - `<C>_<p>_push_data = mux(predicates, values)`
- FIFO pop (consumer’s own `<p>`): `<p>_pop_ready = executed_wire & predicate`
- Async calls: caller outputs `<C>_trigger` (8‑bit sum of call fires). `Top` wires the sum into `<C>_trigger_counter_delta`. Callee sees `trigger_counter_pop_valid` to advance.
- Exposed values: producer emits `expose_<name>` and `valid_<name> = executed_wire`; downstream modules consume `<producer>_<name>` and `<producer>_<name>_valid`.

## Top‑Level Harness

Built by `generate_top_harness`:

- Globals: free‑running `global_cycle_count: Output(UInt(64))` and `global_finish: Output(Bits(1))`.
- SRAMs: per payload array `<a>` allocate `mem_<a>_{address,write_data,write_enable,read_enable,dataout}` wires, instantiate `sramBlackbox_<a>` and connect `dataout`.
- Arrays: instantiate one writer per non‑SRAM array; connect all producers’ write triplets to its ports.
- FIFOs: one FIFO per module input port `<m>.<p>` with `fifo_<m>_<p>_{push_valid,push_data,push_ready,pop_valid,pop_data,pop_ready}` wires; depth is the max explicit `FIFOPush.fifo_depth` across producers (or a small default).
- Trigger counters: one `TriggerCounter` per driver `<m>`, driving `<m>_trigger_counter_{delta,delta_ready,pop_valid,pop_ready}`.
- Instances: all non‑external modules and downstreams are instantiated and connected; unused pushes are tied to zero; `global_finish` is the OR of present `inst_<m>.finish`.

## SRAM Blackboxes

`generate_sram_blackbox_files` emits one `sram_blackbox_<array>.sv` per payload array:

```verilog
module sram_blackbox_<a> #(
    parameter DATA_WIDTH = <bits>,
    parameter ADDR_WIDTH = <bits>
)(
    input clk,
    input [ADDR_WIDTH-1:0] address,
    input [DATA_WIDTH-1:0] wd,
    input banksel,
    input read,
    input write,
    output [DATA_WIDTH-1:0] dataout,
    input rst_n
);
  // Optional $readmemh("<path>", mem) if init file is provided
endmodule
```

Aliases are created (e.g., `fifo_1.sv`) when CIRCT parameterization renames modules in the compiled `Top.sv`.

## Configuration

Common kwargs via `assassyn.backend.elaborate(sys, **kwargs)`:

- `path`: Base output directory. Verilog is placed at `<path>/<sys.name>/verilog`.
- `verilog`: Enable Verilog generation when truthy.
- `sim_threshold`: Max testbench cycles.
- `resource_base`: Base path for SRAM `$readmemh` init files.
- `idle_threshold`, `random`: Simulator‑only (not used by the Verilog backend).
- FIFO depths: inferred from `FIFOPush.fifo_depth`; otherwise default per‑port depth is used.

## Testbench

`tb.py` is a cocotb test that:

- Resets the DUT, then toggles `clk` with a fixed period.
- Prints `Log(...)` messages using predicates translated from module conditions (`valid_*/expose_*` and cycled checks).
- Stops when `dut.global_finish == 1` or `sim_threshold` is reached.
