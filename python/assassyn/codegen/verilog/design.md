# Verilog Design Generation

This module provides the main Verilog design generation functionality, including the CIRCTDumper class that converts Assassyn IR into CIRCT-compatible Verilog code and the generate_design function that orchestrates the complete design generation process. The generator also accumulates the metadata required to stitch together external SystemVerilog modules and multi-port array writers.

Metadata consumed by the dumper (`InteractionMatrix`, `ModuleMetadata`, `ArrayMetadata`,
and the various interaction views) is sourced from the
`python.assassyn.codegen.verilog.metadata` package.  The package re-exports the legacy
symbols while housing their implementations across `metadata.core`, `metadata.module`,
`metadata.array`, and `metadata.fifo`, keeping imports stable for callers.

## Design Documents

- [Simulator Design](../../../docs/design/internal/simulator.md) - Simulator design and code generation
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [External SystemVerilog Integration](../../../docs/design/external/ExternalSV_zh.md) - External module integration
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Verilog Top Generation](./top.md) - Top-level module generation
- [Verilog Elaboration](./elaborate.md) - Main entry point for Verilog generation
- [Simulator Generation](../simulator/simulator.md) - Simulator code generation
- [Module Generation](../simulator/modules.md) - Module-to-Rust translation

## Summary

**Multi-Port Array Architecture Design Decisions:** The Verilog design generation implements a multi-port array architecture with specific design decisions:

1. **Port Arbitration**: Multiple modules can write to the same array through different ports
2. **Port Indexing**: Each write port gets a unique index for compile-time allocation
3. **Port Manager Integration**: Global port manager tracks all port assignments
4. **Conflict Resolution**: Port conflicts are resolved at compile time
5. **Port Metadata**: Port metadata is accumulated for external module integration

**FIFO Handshaking Protocols:** The Verilog design generation implements FIFO handshaking protocols:

1. **Credit-Based Flow Control**: FIFOs use credit-based flow control for backpressure
2. **Valid/Ready Signals**: Standard valid/ready handshaking for data transfer
3. **FIFO Depth Configuration**: FIFO depth is configurable per pipeline stage
4. **FIFO Naming Convention**: FIFOs follow a consistent naming convention
5. **FIFO Integration**: FIFOs are integrated with the credit-based pipeline architecture

**External Module Integration Process:** The Verilog design generation integrates external SystemVerilog modules:

1. **External Module Detection**: External modules are detected during system analysis
2. **FFI Handle Generation**: FFI handles are generated for external module communication
3. **Port Mapping**: External module ports are mapped to internal signals
4. **Signal Routing**: External module signals are routed through the design
5. **Co-simulation Support**: External modules are integrated for co-simulation
6. **Cross-Module Output Tracking**: `CIRCTDumper.external_metadata` stores every cross-module read of an external register output alongside the owning module and instance, and the dumper reuses the normalised wire keys returned by `get_external_wire_key` to declare data/valid ports exactly once per producer.
7. **Centralised Detection Logic**: External-module identification is now handled by the system-analysis bookkeeping shared with the simulator’s [external stub utilities](../simulator/external.md); the dumper does not expose dedicated helper predicates anymore.

**Credit-Based Pipeline Implementation Details:** The Verilog design generation implements the credit-based pipeline:

1. **Pipeline Stage Communication**: Pipeline stages communicate through event queues
2. **Credit Management**: Credits are managed for flow control
3. **Trigger Counter Plumbing**: Trigger counters are plumbed through the design
4. **Stage Register Management**: Stage registers are managed for pipeline state
5. **Asynchronous Call Handling**: Asynchronous calls are handled through the pipeline

**Function Name Inconsistencies (Documented as Potential Improvements):** The Verilog design generation has some function names that don't match their actual implementation:

1. **`cleanup_post_generation`**: Actually generates signal routing, not cleanup
2. **`generate_sram_blackbox_files`**: Generates both blackbox and regular modules
3. **`dump_rval`**: Actually generates signal references, not just values

These inconsistencies are documented as potential improvements for future refactoring.

## Exposed Interfaces

### `generate_design`

```python
def generate_design(fname: Union[str, Path], sys: SysBuilder):
    """Generate a complete Verilog design file for the system."""
```

**Explanation**

This function generates a complete Verilog design file for an Assassyn system. It performs the following steps:

1. **File Setup**: Opens the output file and writes the standard CIRCT header
2. **SRAM Module Generation**: Generates SRAM blackbox module definitions for each SRAM in the system
3. **System Processing**: Uses CIRCTDumper to visit and generate code for all modules in the system
4. **Code Output**: Writes the generated code to the file
5. **Log Return**: Returns the generated log statements for testbench integration

The function handles SRAM modules specially by:
- Extracting SRAM parameters (data width, address width, array name)
- Generating parameterized SRAM blackbox modules
- Creating proper memory interface definitions

**Project-specific Knowledge Required**:
- Understanding of [system builder](/python/assassyn/builder.md)
- Knowledge of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Understanding of [CIRCTDumper integration](/python/assassyn/codegen/verilog/design.md)
- Reference to [Verilog elaboration process](/python/assassyn/codegen/verilog/elaborate.md)

## Internal Classes

### `CIRCTDumper`

```python
class CIRCTDumper(Visitor):
    """Dumps IR to CIRCT-compatible Verilog code."""
```

**Explanation**

The CIRCTDumper class is the main visitor that converts Assassyn IR into Verilog code. It inherits from the Visitor pattern and implements the credit-based pipeline architecture. The class maintains extensive state for managing:

1. **Execution Control**: `wait_until` and per-expression `meta_cond` metadata decide when statements run, while FINISH gating now reads the precomputed `finish_sites` stored in module metadata instead of collecting tuples during emission.
2. **Module State**: `current_module` tracks traversal context, while port declarations are derived from immutable metadata instead of mutating dumper dictionaries.
3. **Array Management**: `array_metadata`, `memory_defs`, and ownership metadata ensure multi-port register arrays are emitted while memory payloads (`array.is_payload(memory)` returning `True`) are routed through dedicated generators.
4. **External Integration**: `external_metadata` (an `ExternalRegistry`) captures external classes, instance ownership, and cross-module reads. Runtime maps (`external_wrapper_names`, `external_instance_names`, `external_wire_assignments`, `external_wire_outputs`, and `external_output_exposures`) reuse that registry to materialise expose/valid ports and wire consumers to producers without recomputing analysis.
5. **Expression Naming**: `expr_to_name` and `name_counters` guarantee deterministic signal names whenever expression results must be reused across statements.
6. **Code Generation**: `code`, `logs`, and `indent` store emitted lines and diagnostic information used later by the testbench.
7. **Module Metadata**: `module_metadata` maps each `Module` to its `ModuleMetadata`. The structure tracks FINISH intrinsics, async calls, FIFO interactions (annotated with `expr.meta_cond`), and every array/value exposure required for cleanup. These entries are populated before the dumper is constructed via [`collect_fifo_metadata`](./analysis.md), so `CIRCTDumper` receives a frozen snapshot and never mutates it during emission. See [metadata module](/python/assassyn/codegen/verilog/metadata.md) for details. The dumper exposes this information via convenience helpers such as `async_callers(module)`, which forwards to the frozen `AsyncLedger` stored on the interaction matrix.

During the cleanup pass the dumper feeds the precomputed metadata into `_emit_predicate_mux_chain`, producing both the `reduce(or_, …)` guards and prioritised mux chains shared by array writes and FIFO pushes. The helper now short-circuits single-entry collections to direct assignments and relies on caller-supplied defaults when metadata yields no interactions, keeping the emitted Verilog stable if predicate formatting or default literals change in the future.

#### Key Methods

**`visit_system`**: Generates code for the entire system by calling `generate_system()`

**`visit_module`**: Generates a complete Verilog module with the following phases:
1. **Analysis Phase**: Assumes module metadata has already been collected. `visit_module` prepares transient state (e.g. code buffers) and processes the module body primarily for code emission; FINISH flags, async calls, and exposure bookkeeping are already locked in the metadata snapshot.
2. **Port Generation**: Calls `generate_module_ports()` to create module interfaces. The helper derives downstream/SRAM/driver roles and reads FIFO plus exposure metadata directly from `CIRCTDumper.module_metadata`, so `visit_module` no longer threads redundant flags or maintains `_exposes`.
3. **Code Integration**: Combines the collected body statements with the module boilerplate and generator decorators.
4. **Special Handling**: Resets external bookkeeping between modules, emits SRAM-specific prelude code, and avoids instantiating pure external stubs.

**`visit_array`**: Generates multi-port register files by delegating to `assassyn.pycde_wrapper.build_register_file`:
- Computes the wrapper name from `array.name` and derives the address width from `array.index_bits` (minimum 1) so single-entry arrays continue to use constant read indices.
- Passes write/read port counts from `ArrayMetadataRegistry`, preserving reverse-priority arbitration for writers through the helper’s internal mux ordering.
- Threads the IR initializer list through to the helper, which coerces values into the target PyCDE element type before constructing the reset literal.
- Requests read-index ports only when the array exposes indexed reads, keeping generated signatures stable for width-one arrays while still wiring `ridx_port<i>` for larger memories.
- The resulting module exposes the same `w*_port<i>`/`widx*_port<i>`/`wdata*_port<i>` and `ridx*_port<i>`/`rdata*_port<i>` interface consumed by `_connect_array`.

**`visit_expr`**: Delegates expression generation to the expression dispatch system, emits helpful `#` comments with source locations, and defers wire reads to the external wiring machinery when applicable. Exposure decisions are made during the analysis pre-pass, so emission only formats code.

**`visit_block`**: Visits conditional and cycled blocks, relying on the IR-level `meta_cond` metadata captured during construction to keep predicates aligned across code generation, metadata collection, and log emission.

**`get_pred(expr)`**: Formats the predicate metadata attached to `expr`. The dumper consumes the final carry exposed via `expr.meta_cond`, and expressions that lack `meta_cond` now trigger an explicit error so refactors cannot silently drop predicate capture.

**`get_external_port_name`**: Creates mangled port names for external values to avoid naming conflicts

**`get_external_wire_key`**: Normalises `(instance, port, index)` access into a hashable key that downstream phases reuse when declaring wires or caching producer exposures, ensuring multi-reader scenarios do not duplicate ports or assignments.

**Project-specific Knowledge Required**:
- Understanding of [visitor pattern](/python/assassyn/ir/visitor.md)
- Knowledge of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Understanding of [module generation](/python/assassyn/codegen/verilog/module.md)
- Reference to [array management](/python/assassyn/codegen/verilog/cleanup.md)
- Knowledge of [external module integration](/python/assassyn/ir/module/external.md)

#### Internal Helpers

**`_generate_external_module_wrapper`**: Creates PyCDE wrapper classes for external SystemVerilog modules. If the external metadata defines explicit wires (and their direction), the wrapper mirrors them; otherwise it falls back to treating the declared ports as inputs for backwards compatibility. Clock/reset ports are emitted when requested by the metadata.

**`_connect_array`**: Handles multi-port array connections between modules by wiring each module’s per-port write enable/data/index signals into the shared register-file instance produced by `build_register_file`. When an array’s address width is zero, the helper omits `ridx_port<i>` entirely, so `_connect_array` only drives the write triplets and surfaces each reader’s data output.

Direct traversal of module bodies is performed inline where needed: since `DONE-remove-block` flattened every module’s `body` list, consumers iterate those statements directly and filter for `Expr` subclasses to perform per-expression analysis.

The CIRCTDumper class integrates with multiple other modules:
- [Expression generation](/python/assassyn/codegen/verilog/_expr/__init__.md) for handling different expression types
- [Cleanup processing](/python/assassyn/codegen/verilog/cleanup.md) for post-generation signal routing
- [Module port generation](/python/assassyn/codegen/verilog/module.md) for interface creation
- [System generation](/python/assassyn/codegen/verilog/system.md) for top-level integration
- [Right-hand value generation](/python/assassyn/codegen/verilog/rval.md) for signal references

**Project-specific Knowledge Required**:
- Understanding of [CIRCT framework](/docs/design/internal/pipeline.md)
- Knowledge of [PyCDE integration](/docs/design/internal/pipeline.md)
- Understanding of [multi-port array architecture](/docs/design/arch/arch.md)
- Reference to [external module system](/python/assassyn/ir/module/external.md)
