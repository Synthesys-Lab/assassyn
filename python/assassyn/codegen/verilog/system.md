# System Generation

This module provides system-level code generation utilities that orchestrate the generation of the complete Verilog system, including module analysis, array management, external module integration, and top-level harness generation.

## Summary

The system generation module is responsible for coordinating the generation of the entire Verilog system from an Assassyn system builder. It performs comprehensive analysis of the system structure, manages array write port assignments, handles external module integration, and orchestrates the generation of all modules and the top-level harness.

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

**Explanation**

This function generates the complete Verilog system by performing comprehensive analysis and orchestration. It executes the following phases:

1. **System Analysis Phase**:
   - **SRAM Payload Identification**: Identifies SRAM payload arrays that need special handling
   - **External Module Collection**: Finds all external modules used in the system
   - **External Module Wrapper Generation**: Creates PyCDE wrapper classes for external modules

2. **Array Management Phase**:
   - **Write Port Assignment**: Assigns unique port indices to each module writing to each array
   - **Array Module Generation**: Generates multi-port array modules for non-SRAM arrays
   - **Array User Analysis**: Identifies which modules read from or write to each array

3. **Module Analysis Phase**:
   - **Expression-to-Module Mapping**: Maps valued expressions to their containing modules
   - **Downstream Dependency Analysis**: Determines dependencies for downstream modules
   - **Async Call Analysis**: Maps async call relationships between modules

4. **Module Generation Phase**:
   - **Regular Module Generation**: Generates code for all non-external modules
   - **Downstream Module Generation**: Generates code for all downstream modules
   - **Top-Level Harness Generation**: Generates the top-level system harness

The function handles complex system-wide relationships:

- **Multi-Port Array Management**: Ensures each array has unique write ports for each writing module
- **External Module Integration**: Properly integrates external SystemVerilog modules
- **Dependency Tracking**: Maintains proper dependency relationships for downstream modules
- **Async Call Relationships**: Tracks which modules call which other modules

**Project-specific Knowledge Required**:
- Understanding of [system builder](/python/assassyn/builder.md)
- Knowledge of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Understanding of [external module integration](/python/assassyn/ir/module/external.md)
- Reference to [array management](/python/assassyn/codegen/verilog/cleanup.md)
- Knowledge of [top-level harness generation](/python/assassyn/codegen/verilog/top.md)

## Internal Helpers

The function uses several analysis and utility functions:

- `get_upstreams()` from [analysis module](/python/assassyn/analysis/external_usage.md) for dependency analysis
- `_is_external_module()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for external module detection
- `_walk_expressions()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for expression traversal
- `_generate_external_module_wrapper()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for external module wrapper generation
- `visit_array()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for array module generation
- `visit_module()` from [CIRCTDumper](/python/assassyn/codegen/verilog/design.md) for module generation
- `generate_top_harness()` from [top module](/python/assassyn/codegen/verilog/top.md) for top-level generation

The function manages several CIRCTDumper state variables:

- `sram_payload_arrays`: Set of arrays that are SRAM payloads
- `external_modules`: List of external modules in the system
- `array_write_port_mapping`: Maps arrays to write port assignments
- `downstream_dependencies`: Maps downstream modules to their dependencies
- `async_callees`: Maps modules to their callers
- `array_users`: Maps arrays to modules that use them

**Project-specific Knowledge Required**:
- Understanding of [CIRCTDumper state management](/python/assassyn/codegen/verilog/design.md)
- Knowledge of [multi-port array architecture](/docs/design/arch/arch.md)
- Understanding of [credit-based pipeline architecture](/docs/design/arch/arch.md)
- Reference to [external module system](/python/assassyn/ir/module/external.md)
- Knowledge of [dependency analysis](/python/assassyn/analysis/external_usage.md)

The system generation is the top-level orchestration function that ensures all components of the Verilog system are properly generated and connected according to the credit-based pipeline architecture and multi-port array design.
