# Backend Module

This module provides the backend interfaces for Assassyn, handling system elaboration, configuration management, and code generation workflows. It serves as the bridge between the high-level IR system and the low-level code generation backends.

---

## Section 1. Exposed Interfaces

This section describes all the function interfaces and data structures in this source file unit that are exposed to the usage for other parts of the project.

### config

```python
def config(path='./workspace', resource_base=None, pretty_printer=True, verbose=True, simulator=True, verilog=False, sim_threshold=100, idle_threshold=100, fifo_depth=4, random=False) -> dict
```

The helper function to create the default configuration for system elaboration. This function provides a centralized way to configure all aspects of the elaboration process.

**Parameters:**
- `path` (str): Base output directory path for generated files (default: './workspace')
- `resource_base` (str, optional): Path to resource files directory
- `pretty_printer` (bool): Whether to run code formatter on generated code (default: True)
- `verbose` (bool): Whether to print verbose output during elaboration (default: True)
- `simulator` (bool): Whether to generate simulator code (default: True)
- `verilog` (bool): Whether to generate Verilog code (default: False)
- `sim_threshold` (int): Maximum simulation cycles before termination (default: 100)
- `idle_threshold` (int): Maximum idle cycles before termination (default: 100)
- `fifo_depth` (int): Default FIFO depth for pipeline stages (default: 4)
- `random` (bool): Whether to randomize module execution order (default: False)

**Returns:**
- A dictionary containing the configuration parameters

**Explanation:**
This function creates a default configuration dictionary that can be customized and passed to the `elaborate` function. It provides sensible defaults for all elaboration parameters while allowing users to override specific settings. The configuration follows the credit-based pipeline architecture described in the [pipeline design document](../../docs/design/internal/pipeline.md).

### make_existing_dir

```python
def make_existing_dir(path) -> None
```

The helper function to create a directory if it does not exist. If the directory already exists, it prints a warning message but continues execution.

**Parameters:**
- `path` (str): Directory path to create

**Returns:**
- None

**Explanation:**
This function safely creates directories for output files during elaboration. It handles the case where directories already exist by printing a warning message, allowing users to be aware of potential file overwrites. This is particularly important in the elaboration workflow where multiple runs might target the same output directory.

### elaborate

```python
def elaborate(sys: SysBuilder, **kwargs) -> List[str]
```

Invoke the elaboration process of the given system, generating simulator and/or Verilog code based on the configuration parameters.

**Parameters:**
- `sys` (SysBuilder): The Assassyn system to be elaborated
- `**kwargs`: Configuration parameters (see `config` function for available options)

**Returns:**
- A list containing `[simulator_manifest, verilog_path]` where:
  - `simulator_manifest`: Path to the generated simulator manifest file (if simulator generation is enabled)
  - `verilog_path`: Path to the generated Verilog directory (if Verilog generation is enabled)

**Explanation:**
This is the main elaboration function that orchestrates the entire code generation process. It performs the following steps:

1. **Configuration Management**: Merges user-provided configuration with default settings, validating all configuration keys
2. **System Inspection**: Prints the system IR if verbose mode is enabled, allowing users to inspect the system before elaboration
3. **Directory Setup**: Creates the output directory structure for the generated files
4. **Code Generation**: Delegates to the `codegen.codegen` function to generate simulator and/or Verilog code
5. **Return Results**: Returns paths to the generated artifacts

The elaboration process follows the module generation principles described in the [module design document](../../docs/design/internal/module.md), translating the high-level IR into executable simulator code and/or synthesizable Verilog code. The function supports both simulation and hardware generation workflows, making it the central entry point for all Assassyn backend operations.

The generated simulator implements the credit-based execution model described in the [simulator design document](../../docs/design/internal/simulator.md), while the Verilog generation follows the pipeline implementation described in the [pipeline design document](../../docs/design/internal/pipeline.md).

---

## Section 2. Internal Helpers

This module contains no internal helper functions or data structures. All functionality is exposed through the public interface.

---

## Usage Pattern

The backend module is typically used as follows:

```python
import assassyn.backend as backend

# Create system
sys = assassyn.SysBuilder("my_system")
# ... build system ...

# Configure elaboration
config = backend.config(
    simulator=True,
    verilog=True,
    sim_threshold=1000,
    idle_threshold=500
)

# Elaborate system
simulator_path, verilog_path = backend.elaborate(sys, **config)
```

This workflow allows users to configure and execute the elaboration process with full control over the generated artifacts and execution parameters.
