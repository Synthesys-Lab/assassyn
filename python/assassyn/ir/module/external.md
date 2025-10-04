# External SystemVerilog Support

External modules let Assassyn integrate pre-existing SystemVerilog blocks directly into a system. The integration is implemented in three layers:

1. Intermediate Representation (IR) nodes that model wire assignments and reads.
2. The `ExternalSV` frontend API that uses those IR nodes to describe the boundary of the foreign module.
3. Verilog code generation that turns the recorded IR into concrete instantiations and wire hookups.
4. TODO we could use verilator to convert external SV into dynamic link which can be used by our rust simulator. 

This document explains first 3 layers and provides an end-to-end example based on `python/ci-tests/test_easy_external.py`.

## IR Nodes for Wire Boundaries

Two expression types defined in `python/assassyn/ir/expr/expr.py` represent external wiring:

- **`WireAssign`** (`python/assassyn/ir/expr/expr.py:740`): stores "drive this wire with that value" relationships. It accepts a `Wire` object and the driving `Value`. These nodes are created through the `wire_assign(wire, value)` helper (decorated with `@ir_builder`) so they are inserted into the IR when builders run.

- **`WireRead`** (`python/assassyn/ir/expr/expr.py:767`): represents "observe this external wire". The `wire_read(wire)` helper creates the node and preserves the data type carried by the wire, allowing the compiler to type-check downstream expressions.

Both expressions inherit from the general `Expr` base class, so the rest of the compiler can analyze them uniformly with other operations.

## ExternalSV Frontend API

`ExternalSV` (`python/assassyn/ir/module/external.py`) extends the base `Module` with a thin façade over external SystemVerilog blocks:

- **Constructor surface**: Alongside the usual `file_path`, `module_name`, and optional `has_clock`/`has_reset` switches, callers spell out the boundary via explicit `in_wires`/`out_wires` dictionaries. Relative file paths are preserved until elaboration resolves them against the build tree.

- **Unified wire adapters**: Declared wires are stored in a single dictionary of `Wire` objects(`python/assassyn/ir/module/module.py`), and lightweight `DirectionalWires` views power both `self.in_wires` and `self.out_wires`. The adapter inspects its configured direction so one class cleanly handles input assignments and output reads.

- **IR integration hooks**: Driving an input—through `self.in_wires[name] = value`, the `in_assign()` helper, constructor keyword arguments, or even `module['a'] = value`—funnels through `wire_assign(...)`, producing a `WireAssign` IR node. Observing an output—by indexing `self.out_wires`, calling `module['c']`, using attribute-style access, or by capturing the return value of `in_assign()`—returns `wire_read(wire_obj)`, ensuring every observation becomes a `WireRead` node. `in_assign()` yields the external outputs in declaration order so callers can unpack them directly.

- **Metadata for downstream stages**: The constructor tags the instance with `Module.ATTR_EXTERNAL` and retains the populated wire dictionary, giving later passes full type/direction information for code emission and validation.

These helpers provide a convenient, side-effect-free API while guaranteeing the IR faithfully records how the external module is wired.

## Code Generation Pipeline

The Verilog backend (`python/assassyn/codegen/verilog/design.py`) consumes those IR breadcrumbs to materialize SystemVerilog instances:

- **Collect input drivers**: As the `CIRCTDumper` walks each block, every `WireAssign` pointing at an `ExternalSV` input is stored in a `pending_external_inputs` table keyed by the owning module.

- **Describe the black box**: `_generate_external_module_wrapper` synthesizes a PyCDE wrapper class whose port list and directions come straight from the stored wire table, aligning the Verilog-facing interface with the frontend description.

- **Instantiate on demand**: The first `WireRead` for a given `ExternalSV` triggers emission of the actual PyCDE instance. The dumper drains the pending input table, threads through optional `clk`/`rst` hookups, and remembers the created instance so subsequent reads simply reference signals like `ext_inst.c`.

## Example: `test_easy_external.py`

`python/ci-tests/test_easy_external.py` demonstrates how to expose a SystemVerilog adder:

1. **Define the external block**:
   ```python
   class ExternalAdder(ExternalModule):
       def __init__(self, **in_wire_connections):
           super().__init__(
               file_path="python/ci-tests/resources/adder.sv",
               module_name="adder",
               in_wires={'a': UInt(32), 'b': UInt(32)},
               out_wires={'c': UInt(32)},
               **in_wire_connections,
           )
   ```
   The `in_wires`/`out_wires` dictionaries set up typed wires and enable the convenience accessors; the optional `**in_wire_connections` passes initial drivers that are turned into `WireAssign` nodes.

2. **Drive inputs and read outputs** inside a downstream module:
   ```python
   c = ext_adder.in_assign(a=a, b=b)
   ```
   `in_assign` records the two input connections via `WireAssign` and returns the single declared output (`WireRead`), making the value immediately usable.

3. **Integrate the external module** just like native modules. We assumed the external module must be instantiated inside the downstream module, which make sure the value align with the wire connection. The system builder instantiates `ExternalAdder()` and passes it into `Adder.build`, allowing the rest of the design to treat `c` as any other value.
