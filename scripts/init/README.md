Scripts to build each component of the project.

## Build System

The build system uses lightweight patches and marker files for efficient incremental builds:

- **Lightweight Patches**: Custom patch format that replaces git patches, avoiding commit hash dependencies
- **Patch Markers**: `.patch-applied` files track when patches have been applied
- **Build Markers**: `.xxx-built` files track when builds are completed, enabling fast rebuilds

## Components

- `py-packages.sh`: Script to install required Python packages for Assassyn.
- `circt.sh`: Script to build the CIRCT Verilog backend.
- `verilator.sh`: Script to build Verilator Verilog simulator.
- `ramulator2.sh`: Script to build the Ramulator2 DRAM simulator.
- `wrapper.sh`: Script to build the Rust wrapper for Ramulator2.

## Patch Format

Patches use a lightweight format:
```
path/to/file
-original line
+replacement line 1
+replacement line 2
```

Features:
- `-` prefix for original lines to be replaced
- `+` prefix for replacement lines
- Exact line matching (whitespace-sensitive)
- Support for multiple replacement lines
- Reverse operation support
