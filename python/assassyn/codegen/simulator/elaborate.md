# Simulator Elaboration

This module provides the main entry point for generating Rust-based simulators from Assassyn systems. It orchestrates the creation of a complete Rust project that can simulate the behavior of Assassyn hardware designs.

## Section 0. Summary

The simulator elaboration process generates a complete Rust project that implements the credit-based pipeline architecture described in the [simulator design document](../../../docs/design/internal/simulator.md). The generated simulator faithfully executes the high-level execution model by translating Assassyn operations into corresponding Rust operations, with special handling for register writes, stage registers, and asynchronous calls.

## Section 1. Exposed Interfaces

### elaborate

```python
def elaborate(sys, **config):
    """Generate a Rust-based simulator for the given Assassyn system.

    This function is the main entry point for simulator generation. It takes
    an Assassyn system builder and configuration options, and generates a Rust
    project that can simulate the system.

    Args:
        sys: The Assassyn system builder
        **config: Refer to ..codegen for the list of options

    Returns:
        Path to the generated Cargo.toml file
    """
```

**Explanation:**

This function orchestrates the complete simulator generation process. It first resets the global port manager to ensure clean state for port assignments, then delegates to `elaborate_impl` for the actual generation work. After generation, it attempts to format the generated Rust code using `cargo fmt` if available.

The function handles the coordination between different components of the simulator generation pipeline, ensuring proper initialization and cleanup of global state. The port manager reset is particularly important for testing scenarios where multiple compilations might be performed in the same process.

## Section 2. Internal Helpers

### elaborate_impl

```python
def elaborate_impl(sys, config):
    """Internal implementation of the elaborate function.

    This matches the Rust function in src/backend/simulator/elaborate.rs
    """
```

**Explanation:**

This function performs the core work of simulator generation. It follows these steps:

1. **Directory Setup**: Creates and optionally cleans the simulator output directory based on the `override_dump` configuration option. The directory structure includes a `src` subdirectory for Rust source files.

2. **Project Configuration**: Generates a `Cargo.toml` manifest file that defines the Rust project with dependencies on the `sim-runtime` crate. The project name is derived from the system name.

3. **Code Generation**: Orchestrates the generation of Rust source files:
   - Calls `dump_modules` to generate the `modules` directory containing individual module implementations
   - Calls `dump_simulator` to generate the main `simulator.rs` file containing the simulator context and execution logic
   - Copies the template `main.rs` file to provide the entry point

4. **Formatting**: Copies the project's `rustfmt.toml` configuration to ensure consistent code formatting.

The function creates a complete, self-contained Rust project that can be compiled and executed to simulate the Assassyn system. The generated simulator implements the credit-based pipeline architecture with proper handling of register arrays, stage registers, and asynchronous module communication as described in the [simulator design document](../../../docs/design/internal/simulator.md).

The implementation matches the behavior of the Rust backend's elaborate function, ensuring consistency between Python and Rust implementations of the simulator generation process.
