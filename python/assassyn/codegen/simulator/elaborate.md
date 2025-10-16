# Simulator Elaboration

This module provides the main entry point for generating Rust-based simulators from Assassyn systems. It orchestrates the creation of a complete Rust project that can simulate the behavior of Assassyn hardware designs.

## Design Documents

- [Simulator Design](../../../docs/design/internal/simulator.md) - Simulator design and code generation
- [Pipeline Architecture](../../../docs/design/internal/pipeline.md) - Credit-based pipeline system
- [Architecture Overview](../../../docs/design/arch/arch.md) - Overall system architecture
- [Build System](../../../docs/design/internal/build-system.md) - Build and elaboration system

## Related Modules

- [Simulator Generation](./simulator.md) - Core simulator generation logic
- [Module Generation](./modules.md) - Module-to-Rust translation
- [Node Dumper](./node_dumper.md) - IR node reference generation
- [Port Mapper](./port_mapper.md) - Multi-port array write support

## Section 0. Summary

The simulator elaboration process generates a complete Rust project that implements the credit-based pipeline architecture described in the [simulator design document](../../../docs/design/internal/simulator.md). Besides translating Assassyn operations into Rust with proper handling for register writes, pipeline stage scheduling, and asynchronous calls, the elaborator now understands external SystemVerilog FFIs. During elaboration we emit stub crates for every external SV binding, add them as workspace dependencies, and surface the FFI handles in the generated simulator so that Rust code can drive co-simulated peripherals.

**Python-Rust Consistency Requirements:** The elaboration process ensures consistency between Python and Rust implementations:

1. **Data Type Mapping**: Assassyn data types are mapped to corresponding Rust types:
   - `UInt` → `u32` or `u64` (based on width)
   - `Bits` → `bool`
   - `Record` → Rust struct with named fields
   - `Array` → Rust array or Vec

2. **Memory Interface**: DRAM interfaces use the same request/response protocol as the Python implementation

3. **FIFO Naming**: FIFO names follow the same convention as the Python implementation

4. **Module Execution**: Module execution order and timing must match the Python simulation model

5. **External FFI Integration**: External SystemVerilog modules are integrated consistently with the Python co-simulation approach

**Code Generation Pipeline Dependencies:** The elaboration process follows a specific pipeline:

1. **System Analysis**: Analyze the Assassyn system to identify modules, arrays, and external dependencies
2. **Port Registration**: Register all array write ports with the port manager
3. **DRAM Collection**: Collect all DRAM modules for memory interface generation
4. **External FFI Processing**: Process external SystemVerilog FFI specifications
5. **Rust Project Generation**: Generate the complete Rust project structure
6. **Simulator Code Generation**: Generate the simulator implementation code
7. **External Module Integration**: Integrate external SystemVerilog modules

**Project-Specific Knowledge:** The elaboration process requires understanding of:

1. **Bit Manipulation**: How Assassyn bit operations map to Rust bit operations
2. **Cycle Counting**: How simulation cycles are tracked and managed
3. **FIFO Naming**: The naming convention for FIFOs in the generated code
4. **Assassyn-Rust Type Mapping**: How Assassyn types are mapped to Rust types
5. **External Module Integration**: How external SystemVerilog modules are integrated

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

This public entry point orchestrates the complete simulator generation process. It first resets the global port manager (via `reset_port_manager`) so array port numbering starts from a clean state, delegates the heavy lifting to `elaborate_impl`, and finally makes a best-effort `cargo fmt` run over the generated crate. Formatting failures (missing cargo or fmt errors) are downgraded to warnings so pipelines can keep moving.

The wrapper is intentionally thin so that doctests and unit tests can call `elaborate_impl` directly while still keeping the global state reset/formatting behaviour available to CLI users.

### _write_manifest

```python
def _write_manifest(simulator_path: Path, sys_name: str, ffi_specs) -> Path:
    """Write the Cargo manifest for the generated simulator crate."""
```

**Explanation:**

This helper writes `Cargo.toml` into the simulator directory. In addition to the fixed `sim-runtime` dependency (resolved via a relative path inside the repository) it now iterates over `ffi_specs`, wiring every generated external SystemVerilog bridge crate into the manifest using paths relative to the simulator root. Returning the manifest path keeps the helper easy to test and lets callers feed it straight into `cargo fmt`.

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

1. **Directory Setup**: Derives the output paths (simulator root and optional Verilator workspace), removes the simulator directory when `override_dump` is `True`, and ensures `src/` exists.

2. **External FFI Discovery**: Calls `emit_external_sv_ffis` to synthesise Rust crates that wrap every `ExternalSV` module used by the system. The helper returns `ffi_specs`, which describe crate names, on-disk locations, and whether a clocked callback is required.

3. **Project Configuration**: Invokes `_write_manifest` so the generated Cargo manifest depends on `sim-runtime` and all FFI crates. The project name is derived from `sys.name`, and `rustfmt.toml` is copied alongside the manifest so formatting is deterministic.

4. **Code Generation**: Orchestrates the generation of Rust source files:
   - Calls `dump_modules` to generate the `modules` directory with per-module implementations (including DRAM callbacks and external handle stubs)
   - Calls `dump_simulator` to generate `src/simulator.rs`, passing the configuration so that simulator state mirrors the available externals
   - Copies the pre-baked `main.rs` template that wires everything into a runnable binary

5. **Return Value**: Propagates the manifest path so callers can chain further tooling (formatters, builds, or tests) without recomputing the location.

The implementation mirrors the Rust backend (see `src/backend/simulator/elaborate.rs`) so that both code paths share behaviour: array port allocation, DRAM response plumbing, and external FFI visibility all match the canonical simulator runtime.
