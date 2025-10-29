# System Generation

This module provides system-level code generation utilities that orchestrate the generation of the complete Verilog system, including module analysis, array management, external module integration, trigger bookkeeping, and top-level harness generation.

## Summary

The system generation module is responsible for coordinating the generation of the entire Verilog system from an Assassyn system builder. It performs comprehensive analysis of the system structure, manages array write port assignments, handles external module integration (including FFI wiring), and orchestrates the generation of all modules and the top-level harness.

## Exposed Interfaces

### `generate_system`

```python
def generate_system(dumper, node):
    """Generate code for the entire system.

    Args:
        dumper: The CIRCTDumper instance
        node: The SysBuilder instance to generate code for
    """
```
Note on typing:

- The function parameter `dumper` is annotated as `CIRCTDumper` for static type checking.
- To avoid a runtime cyclic import between `system.py` and `design.py`, the module uses a `TYPE_CHECKING` guard and aliases `CIRCTDumper` to `typing.Any` at runtime. This preserves type information for static analyzers (mypy/pylance) while breaking the cycle during execution.


**Explanation**

This function generates the complete Verilog system by performing comprehensive analysis and orchestration. It executes the following phases:

1. **System Analysis Phase**:
   - **SRAM Payload Identification**: Identifies SRAM payload arrays that need special handling.
   - **External Module Collection**: Harvests every `ExternalIntrinsic` in the system, records per-instance metadata, and generates PyCDE wrapper classes for each unique external class upfront.
   - **Cross-Module External Reads**: Scans every module body for `PureIntrinsic.EXTERNAL_OUTPUT_READ` operations whose producer lives in a different module, storing both the consumer-facing entries (`cross_module_external_reads`) and the producer-facing grouping (`external_outputs_by_instance`). Producer lookup first checks whether the intrinsic’s parent is already a module—supporting the block-free IR—before falling back to legacy `.module` handles when present.

2. **Array Management Phase**:
   - **Write Port Assignment**: Assigns unique port indices to each module writing to an array, recording them inside `dumper.array_metadata`.
   - **Array Module Generation**: Generates multi-port array modules for non-SRAM arrays via `visit_array`.
   - **Array User Analysis**: Populates the registry with every module that reads or writes each array by iterating the flattened `module.body` lists directly, so downstream passes can query a single source of truth without relying on dumper-specific helpers.

3. **Module Analysis Phase**:
   - **Dependency Tracking**: Records downstream dependencies using `get_upstreams`.
   - **Async Call Analysis**: Fills `dumper.async_callees` so trigger counters can sum incoming credits.
   - **External Wiring**: Records which exposed values flow across module boundaries so the top-level harness can declare and route the corresponding wires; legacy `external_wire_assignments` have been retired in favour of the intrinsic-driven bookkeeping, which now includes both consumer-side port declarations and producer-side exposure planning.

4. **Module Generation Phase**:
   - **Regular Module Generation**: Generates code for all recorded modules; pure external stubs are filtered out earlier when collecting external intrinsic metadata.
   - **Downstream Module Generation**: Processes downstream modules after regular modules.
   - **Top-Level Harness Generation**: Marks `is_top_generation`, invokes `generate_top_harness`, then resets the flag.

The function handles complex system-wide relationships:

- **Multi-Port Array Management**: Ensures each array has unique write ports for each writing module and that shared array writer modules are emitted before they are referenced.
- **External Module Integration**: Tracks which values need to cross between producers and external consumers, and records the wiring information needed by the top-level harness.
- **Dependency Tracking**: Maintains proper dependency relationships for downstream modules.
- **Async Call Relationships**: Tracks which modules call which other modules so trigger counters can aggregate requests.

**Project-specific Knowledge Required**:
- Understanding of [system builder](/python/assassyn/builder.md)
- Knowledge of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Understanding of [external module integration](/python/assassyn/ir/module/external.md)
- Reference to [array management](/python/assassyn/codegen/verilog/cleanup.md)
- Knowledge of [top-level harness generation](/python/assassyn/codegen/verilog/top.md)

## Internal Helpers

The function uses several analysis and utility functions:

- `get_upstreams()` from [analysis module](/python/assassyn/analysis/external_usage.md) for dependency analysis
- `_generate_external_module_wrapper()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for external module wrapper generation
- `visit_array()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for array module generation
- `visit_module()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for module generation
- `generate_top_harness()` from [top module](/python/assassyn/codegen/verilog/top.md) for top-level generation

Expression traversal now happens inline: each analysis iterates the flattened `module.body` list directly and filters entries to `Expr` instances. This preserves ordering while avoiding a dedicated dumper helper.

The function manages several CIRCTDumper state variables:

- `external_intrinsics`: List of `ExternalIntrinsic` nodes encountered in the system
- `external_classes`: Unique set of external classes that require PyCDE wrappers
- `cross_module_external_reads`: Consumer-side records of external register outputs read from another module
- `external_outputs_by_instance`: Producer-side grouping of the external outputs that must be exposed for other modules
- `external_output_exposures`: Per-module cache populated during instantiation to drive `cleanup_post_generation`
- `array_metadata`: `ArrayMetadataRegistry` instance with write/read port assignments and user membership. Arrays whose owner is a memory instance and satisfy `array.is_payload(owner)` are skipped during collection because they are handled by dedicated memory generators.
- `downstream_dependencies`: Maps downstream modules to their dependencies
- `async_callees`: Maps modules to their callers

**Project-specific Knowledge Required**:
- Understanding of [CIRCTDumper state management](/python/assassyn/codegen/verilog/design.md)
- Knowledge of [multi-port array architecture](/docs/design/arch/arch.md)
- Understanding of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Reference to [external module system](/python/assassyn/ir/module/external.md)
- Knowledge of [dependency analysis](/python/assassyn/analysis/external_usage.md)

The system generation is the top-level orchestration function that ensures all components of the Verilog system are properly generated and connected according to the credit-based pipeline architecture and multi-port array design.
