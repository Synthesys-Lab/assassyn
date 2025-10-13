# Testbench Generation

This module provides testbench generation utilities for Verilog simulation, creating Cocotb-based testbenches that can be used to verify the generated Verilog designs.

## Summary

The testbench generation module creates Python-based testbenches using the Cocotb framework for Verilog simulation. It generates testbenches that include clock/reset sequences, simulation control, logging integration, and proper source file management for the simulation environment.

## Exposed Interfaces

### `generate_testbench`

```python
def generate_testbench(fname: str, _sys: SysBuilder, sim_threshold: int,
                       dump_logger: List[str], external_files: List[str]):
    """Generate a testbench file for the given system."""
```

**Explanation**

This function generates a complete Cocotb-based testbench for Verilog simulation. It performs the following steps:

1. **Template Processing**: Uses a predefined template to generate the testbench code
2. **Log Integration**: Embeds the generated log statements from the design generation
3. **Source File Management**: Includes all necessary source files for simulation
4. **Simulation Control**: Sets up proper simulation parameters and control flow

The generated testbench includes:

- **Cocotb Test Function**: `test_tb()` function that implements the main test logic
- **Clock/Reset Sequence**: Proper initialization sequence with clock and reset signals
- **Simulation Loop**: Main simulation loop that runs for the specified threshold
- **Log Integration**: Embedded logging statements from the design generation
- **Finish Detection**: Early termination when the global finish signal is asserted
- **Runner Function**: Cocotb runner configuration for Verilator simulation

The testbench template handles:

- **Clock Generation**: 1ns period clock with proper timing
- **Reset Sequence**: Active-high reset for 500ns followed by normal operation
- **Simulation Control**: Runs for the specified number of cycles or until finish
- **Source File Management**: Includes all necessary Verilog source files
- **External File Support**: Includes additional external SystemVerilog files

**Project-specific Knowledge Required**:
- Understanding of [Cocotb framework](https://docs.cocotb.org/) for Python-based verification
- Knowledge of [Verilator simulation](/docs/design/internal/pipeline.md)
- Understanding of [testbench integration](/python/assassyn/codegen/verilog/design.md)
- Reference to [logging system](/python/assassyn/codegen/verilog/_expr/intrinsics.md)

## Internal Constants

### `TEMPLATE`

The `TEMPLATE` constant contains the complete Cocotb testbench template with placeholders for:

- **Simulation Threshold**: `{}` - Maximum number of simulation cycles
- **Log Statements**: `{}` - Generated log statements from the design
- **External Files**: `{}` - Additional external SystemVerilog files

The template includes:

1. **Imports**: Cocotb framework imports and utilities
2. **Test Function**: Main test function with clock/reset sequence
3. **Simulation Loop**: Cycle-based simulation with logging and finish detection
4. **Runner Configuration**: Verilator-based simulation setup
5. **Source File Management**: Automatic inclusion of all necessary source files

The template generates a complete testbench that:
- Uses Verilator as the simulation backend
- Includes all generated Verilog source files
- Supports SRAM blackbox modules
- Includes FIFO and trigger counter templates
- Supports external SystemVerilog modules
- Provides proper simulation control and logging

**Project-specific Knowledge Required**:
- Understanding of [Verilator simulation](/docs/design/internal/pipeline.md)
- Knowledge of [FIFO and trigger counter templates](/docs/design/internal/pipeline.md)
- Understanding of [SRAM blackbox integration](/python/assassyn/ir/memory/sram.md)
- Reference to [external module system](/python/assassyn/ir/module/external.md)

The testbench generation is integrated into the Verilog elaboration process through the [elaborate module](/python/assassyn/codegen/verilog/elaborate.md), which calls this function to generate the testbench after the main design generation is complete.
