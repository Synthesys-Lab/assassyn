# Top-Level Harness Generation

This module provides top-level harness generation for Verilog designs, creating the complete system-level module that instantiates and connects all components including modules, arrays, FIFOs, trigger counters, and external modules.

## Design Documents

- [Simulator Design](../../../docs/design/internal/simulator.md) - Simulator design and code generation
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [External SystemVerilog Integration](../../../docs/design/external/ExternalSV_zh.md) - External module integration
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Verilog Design Generation](./design.md) - Core Verilog design generation
- [Verilog Elaboration](./elaborate.md) - Main entry point for Verilog generation
- [Simulator Generation](../simulator/simulator.md) - Simulator code generation
- [Module Generation](../simulator/modules.md) - Module-to-Rust translation

## Summary

The top-level harness generation module creates the complete system-level Verilog module that serves as the top-level of the design. It handles the instantiation and connection of all system components, including regular modules, downstream modules, SRAM modules, multi-port arrays, FIFOs, trigger counters, and external modules, while managing the complex interconnections required by the credit-based pipeline architecture.

Metadata lookups referenced throughout (array usage, module interactions, FIFO
projections) flow from the `python.assassyn.codegen.verilog.metadata` package, whose
submodules (`metadata.core`, `metadata.array`, `metadata.module`, `metadata.fifo`)
provide the concrete implementations behind the familiar `metadata` namespace imports.

## Exposed Interfaces

### `generate_top_harness`

```python
def generate_top_harness(dumper):
    """
    Generates a generic Top-level harness that connects all modules based on
    the analyzed dependencies (async calls, array usage).
    """
```

**Explanation**

This function generates the complete top-level Verilog module that serves as the system's root. It performs the following comprehensive steps:

1. **Top Module Declaration**: Creates the `Top` class with standard system ports:
   - `clk = Clock()`: System clock
   - `rst = Reset()`: System reset
   - `global_cycle_count = Output(UInt(64))`: Global cycle counter for testbench
   - `global_finish = Output(Bits(1))`: Global finish signal

2. **SRAM Memory Blackbox Instantiation**: For each SRAM module:
   - Generates memory interface wires (dataout, address, write_data, write_enable, read_enable)
   - Instantiates SRAM blackbox modules with proper connections
   - Connects memory interfaces to the blackbox instances

3. **Global Cycle Counter**: Creates a free-running counter for testbench control

4. **Wire Declarations**: Generates wires for all system interconnections:
   - **FIFO Wires**: Push/pop valid, data, and ready signals for each FIFO
   - **Trigger Counter Wires**: Delta, ready, and valid signals for each module
   - **Array Wires**: Write enable, data, and address signals for multi-port arrays

5. **Hardware Instantiations**: Instantiates all system components:
   - **FIFO Instances**: Parameterized FIFOs with depth configuration derived from metadata
   - **Trigger Counter Instances**: Credit-based trigger counters for each module
   - **Array Instances**: Multi-port array modules with write port connections

6. **Module Instantiations**: Instantiates all modules with proper port connections:
   - **Regular Modules**: Connected to trigger counters and FIFO ports
   - **Downstream Modules**: Connected to dependency signals and external values
   - **SRAM Modules**: Connected to memory interfaces
   - **External Modules**: Hooked up through helper routines that splice in cross-module wires derived from both `module.externals` and the cross-module metadata precomputed during system analysis, and avoid duplicating instantiations. Producer discovery now checks whether `expr.parent` is already a module—reflecting the block removal refactor—before consulting legacy `.module` fields so mixed IR shapes continue to work.

7. **Module Connections**: Creates all inter-module connections:
   - **FIFO Connections**: Push/pop signal routing between modules
   - **Trigger Connections**: Async call trigger signal routing
   - **Array Connections**: Write signal routing to array instances
   - **Memory Connections**: SRAM interface signal routing

8. **Global Finish Signal**: Collects finish signals from all modules using metadata-based detection (O(1) lookup via `module_metadata`), avoiding redundant expression walking, and creates global finish. See [metadata module](/python/assassyn/codegen/verilog/metadata.md) for implementation details.
  Additionally, FIFO depth selection no longer walks expressions; it computes depths from the predicated push expressions stored in the shared `InteractionMatrix` (queried via `dumper.interactions.fifo_view(port).pushes`), with `module_metadata.interactions.pushes` serving as the module-scoped projection of the same tuples.

9. **Unused Port Tie-off**: Ties off unused FIFO push ports to prevent floating signals

10. **Array Write-back Connections**: Connects array write signals back to array instances

11. **Trigger Counter Delta Connections**: Routes trigger signals to trigger counters

12. **System Compilation**: Creates the PyCDE system and compiles it

The function handles complex system-wide relationships:

- **Multi-Port Array Management**: Ensures proper write port assignment and connection
- **FIFO Depth Configuration**: Determines FIFO depths from `dumper.interactions.fifo_view(port).pushes` (no expression walking, predicate context preserved for downstream analysis, and the data is mirrored by the module-scoped interaction view)
- **External Module Integration**: Properly integrates external SystemVerilog modules
  by:
  - Declaring shared wires once per exposed external value (data + valid), using the normalised wire keys emitted by the intrinsic lowering pass
  - Routing assignments based on the intrinsic-derived `node.externals` list and the additional consumer wiring recorded in `external_metadata.cross_module_reads` so instantiations stay in emission order and each producer drives a single pair of signals
- **Dependency Management**: Handles downstream module dependencies
- **Credit-based Pipeline**: Implements proper trigger counter and credit management

**Project-specific Knowledge Required**:
- Understanding of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Knowledge of [multi-port array architecture](/docs/design/arch/arch.md)
- Understanding of [FIFO handshaking protocols](/docs/design/internal/pipeline.md)
- Reference to [SRAM memory interface](/python/assassyn/ir/memory/sram.md)
- Knowledge of [external module integration](/python/assassyn/ir/module/external.md)
- Understanding of [topological ordering](/python/assassyn/analysis/external_usage.md)

## Internal Helpers

The function uses several utility functions and data structures:

- `dump_type()` and `dump_type_cast()` from [utils module](/python/assassyn/codegen/verilog/utils.md) for type handling
- `get_sram_info()` from [utils module](/python/assassyn/codegen/verilog/utils.md) for SRAM information
- `namify()` and `unwrap_operand()` from [utils module](/python/assassyn/utils.md) for name generation
- `topo_downstream_modules()` and `get_upstreams()` from [analysis module](/python/assassyn/analysis/external_usage.md) for topological ordering and dependency discovery
- `get_external_port_name()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for external port naming
- Metadata-driven checks for `FIFOPop` readiness: the predicated pop entries surfaced from `module_metadata.interactions.pops` (backed by the same tuples returned from `dumper.interactions.fifo_view(port).pops`) determine whether `<port>_pop_ready` connections should be emitted, replacing the legacy dumper helper traversal.
- `_connect_array()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for array connections

The function manages several CIRCTDumper state variables:

- `memory_defs`: SRAM memory definitions
- `array_metadata`: Registry containing array write/read port assignments and usage
- `async_callers(module)`: Helper forwarding to the async-call ledger for trigger aggregation and driver detection

**Project-specific Knowledge Required**:
- Understanding of [CIRCTDumper state management](/python/assassyn/codegen/verilog/design.md)
- Knowledge of [PyCDE system compilation](/docs/design/internal/pipeline.md)
- Understanding of [FIFO and trigger counter templates](/docs/design/internal/pipeline.md)
- Reference to [external module system](/python/assassyn/ir/module/external.md)
- Knowledge of [dependency analysis](/python/assassyn/analysis/external_usage.md)

The top-level harness generation is the final step in the Verilog code generation process, creating the complete system-level module that integrates all components and implements the credit-based pipeline architecture with proper interconnections and control flow.
