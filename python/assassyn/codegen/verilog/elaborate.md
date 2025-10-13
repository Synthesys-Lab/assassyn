# Verilog Elaboration

This module provides the main elaboration function for Verilog code generation, orchestrating the complete process of converting Assassyn IR into synthesizable Verilog code with testbenches, external module integration, and resource file management.

## Summary

The Verilog elaboration module is the main entry point for Verilog code generation, coordinating the complete process of converting an Assassyn system into synthesizable Verilog code. It handles design generation, testbench creation, external module integration, resource file management, and SRAM blackbox generation.

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

1. **Directory Setup**: Creates and cleans the output directory structure
2. **External Module Analysis**: Identifies external SystemVerilog modules and their source files
3. **Design Generation**: Calls `generate_design()` to create the main Verilog design
4. **Testbench Generation**: Calls `generate_testbench()` to create the simulation testbench
5. **Resource File Management**: Copies and manages all necessary resource files
6. **SRAM Blackbox Generation**: Creates SRAM memory blackbox modules
7. **External File Integration**: Copies external SystemVerilog files to the output directory

The function handles complex file management:

- **Resource File Copying**: Copies FIFO and trigger counter templates
- **Alias File Creation**: Creates aliased versions of parameterized modules
- **External File Integration**: Copies external SystemVerilog modules
- **SRAM Blackbox Generation**: Creates memory blackbox modules with initialization

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

1. **SRAM Analysis**: Identifies all SRAM modules in the system
2. **Parameter Extraction**: Extracts SRAM parameters (data width, address width, array name)
3. **Verilog Generation**: Creates SystemVerilog blackbox modules with:
   - Parameterized data and address widths
   - Clock and reset interfaces
   - Read/write control signals
   - Memory array declaration
   - Initialization support (if init_file is specified)
   - Read/write logic implementation

The generated SRAM blackbox modules include:

- **Parameterized Interface**: Configurable data and address widths
- **Memory Array**: Register-based memory with configurable depth
- **Initialization Support**: Optional memory initialization from files
- **Read/Write Logic**: Proper read/write control with bank selection
- **Reset Logic**: Memory initialization on reset (if no init file)

**Project-specific Knowledge Required**:
- Understanding of [SRAM memory model](/python/assassyn/ir/memory/sram.md)
- Knowledge of [SRAM parameter extraction](/python/assassyn/codegen/verilog/utils.md)
- Understanding of [SystemVerilog memory interfaces](/docs/design/internal/pipeline.md)

## Internal Helpers

The module uses several utility functions:

- `extract_sram_params()` from [utils module](/python/assassyn/codegen/verilog/utils.md) for SRAM parameter extraction
- `create_and_clean_dir()` from [utils module](/python/assassyn/utils.md) for directory management
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
