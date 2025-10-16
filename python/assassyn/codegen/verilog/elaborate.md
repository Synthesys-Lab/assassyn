# Verilog Elaboration

This module provides the main elaboration function for Verilog code generation, orchestrating the complete process of converting Assassyn IR into synthesizable Verilog code with testbenches, external module integration, SRAM blackbox emission, and resource file management.

## Design Documents

- [Simulator Design](../../../docs/design/internal/simulator.md) - Simulator design and code generation
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [External SystemVerilog Integration](../../../docs/design/external/ExternalSV_zh.md) - External module integration
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture

## Related Modules

- [Verilog Design Generation](./design.md) - Core Verilog design generation
- [Verilog Top Generation](./top.md) - Top-level module generation
- [Simulator Generation](../simulator/simulator.md) - Simulator code generation
- [Module Generation](../simulator/modules.md) - Module-to-Rust translation

## Summary

The Verilog elaboration module is the main entry point for Verilog code generation, coordinating the complete process of converting an Assassyn system into synthesizable Verilog code. It handles design generation, testbench creation, external module integration, SRAM blackbox generation, alias resource synthesis, and general resource file management.

## Exposed Interfaces

### `elaborate`

```python
def elaborate(sys: SysBuilder, **kwargs) -> str:
    """Elaborate the system into Verilog.

    Args:
        sys: The system to elaborate
        **kwargs: Configuration options including:
            - verilog: The simulator to use ("Verilator", "VCS", or None)
            - resource_base: Path to resources
            - override_dump: Whether to override existing files
            - sim_threshold: Simulation threshold
            - idle_threshold: Idle threshold
            - random: Whether to randomize execution
            - fifo_depth: Default FIFO depth

    Returns:
        Path to the generated Verilog files
    """
```

**Explanation**

This function is the main entry point for Verilog code generation, orchestrating the complete elaboration process. It performs the following comprehensive steps:

1. **Directory Setup**: Resolves the output directory (default `<cwd>/verilog`), ensures it exists, and optionally wipes prior results when `override_dump` is set.
2. **External Module Analysis**: Collects the source files referenced by `ExternalSV` modules so they can be copied alongside the generated design.
3. **Design Generation**: Calls `generate_design()` to build `design.py` and capture log metadata for the testbench.
4. **Alias Discovery**: If a previous `Top.sv` exists, scans it for parameterised module aliases (e.g. `fifo_1`) so matching resource files can be cloned.
5. **Testbench Generation**: Calls `generate_testbench()` with the discovered alias list and external file names, ensuring the Cocotb harness imports every required HDL artifact.
6. **SRAM Blackbox Generation**: Invokes `generate_sram_blackbox_files()` so each SRAM downstream module receives a behavioural blackbox wrapper.
7. **Resource File Management**: Copies core support files (`fifo.sv`, `trigger_counter.sv`), materialises alias copies when required, and copies user-supplied SystemVerilog sources (resolving relative paths via `repo_path()`).

The function handles complex file management:

- **Resource File Copying**: Copies FIFO and trigger counter templates into the output directory.
- **Alias File Creation**: Clones template resources under alias names when CIRCT produces suffixed module instances.
- **External File Integration**: Copies external SystemVerilog modules (absolute or repository-relative) into the output tree.
- **SRAM Blackbox Generation**: Emits behavioural SRAM wrappers with optional `readmemh` initialisation.

**Project-specific Knowledge Required**:
- Understanding of [system builder](/python/assassyn/builder.md)
- Knowledge of [design generation](/python/assassyn/codegen/verilog/design.md)
- Understanding of [testbench generation](/python/assassyn/codegen/verilog/testbench.md)
- Reference to [external module integration](/python/assassyn/ir/module/external.md)
- Knowledge of [SRAM memory model](/python/assassyn/ir/memory/sram.md)

### `generate_sram_blackbox_files`

```python
def generate_sram_blackbox_files(sys, path, resource_base=None):
    """Generate separate Verilog files for SRAM memory blackboxes."""
```

**Explanation**

This function generates separate Verilog files for SRAM memory blackbox modules. It performs the following steps:

1. **SRAM Analysis**: Identifies all SRAM downstream modules in the system and obtains their payload metadata via `extract_sram_params`.
2. **Template Emission**: Writes a SystemVerilog module per SRAM that declares the memory, clock/reset, address/data ports, and banksel/read/write controls.
3. **Initialisation Support**: When the SRAM metadata specifies an `init_file`, emits an `initial begin $readmemh(...); end` block using either the provided `resource_base` directory or the raw path.
4. **Reset Behaviour**: For SRAMs without an init file, generates reset logic that clears the memory contents when `rst_n` is asserted low.
5. **Read/Write Logic**: Implements simple synchronous write behaviour guarded by `write & banksel` and combinational readback when `read & banksel` is asserted.

The generated wrappers provide a behavioural memory model suitable for simulation while keeping the interface parameterised so integrators can replace them with technology-specific implementations if required.

**Project-specific Knowledge Required**:
- Understanding of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Knowledge of [SRAM parameter extraction](/python/assassyn/codegen/verilog/utils.md)
- Understanding of [SystemVerilog memory interfaces](/docs/design/internal/pipeline.md)

## Internal Helpers

The module uses several utility functions:

- `extract_sram_params()` from [utils module](/python/assassyn/codegen/verilog/utils.md) for SRAM parameter extraction
- `create_dir()` from [utils module](/python/assassyn/utils.md) for directory management
- `repo_path()` from [utils module](/python/assassyn/utils.md) for repository path resolution
- `generate_design()` from [design module](/python/assassyn/codegen/verilog/design.md) for main design generation
- `generate_testbench()` from [testbench module](/python/assassyn/codegen/verilog/testbench.md) for testbench generation

The elaboration process handles several file types:

- **Design Files**: Main Verilog design generated by CIRCTDumper
- **Testbench Files**: Python-based Cocotb testbenches
- **Resource Files**: FIFO and trigger counter templates
- **External Files**: User-provided SystemVerilog modules
- **SRAM Files**: Generated memory blackbox modules
- **Alias Files**: Parameterized module aliases

**Project-specific Knowledge Required**:
- Understanding of [CIRCT framework](/docs/design/internal/pipeline.md)
- Knowledge of [Cocotb testbench framework](https://docs.cocotb.org/)
- Understanding of [external module system](/python/assassyn/ir/module/external.md)
- Reference to [resource file management](/docs/design/internal/pipeline.md)
- Knowledge of [file path resolution](/python/assassyn/utils.md)

The Verilog elaboration is the main entry point for the Verilog backend, called from the [codegen implementation](/python/assassyn/codegen/impl.md) to generate complete Verilog designs with all necessary components for simulation and synthesis.
