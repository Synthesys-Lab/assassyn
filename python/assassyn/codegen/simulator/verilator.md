# Verilator FFI Generation

`verilator.py` owns the pipeline that turns every `ExternalSV` module into a self-contained Verilator-backed Cargo crate. The generated artifacts are later linked into the Rust simulator so foreign SystemVerilog blocks can execute beside native Assassyn modules.

## Section 0. Summary

For each external block the generator:

1. Creates a dedicated crate directory (`verilated_<name>`).
2. Copies the source `.sv` into `rtl/`.
3. Writes a Rust wrapper (`src/lib.rs`) and a C++ shim (`src/wrapper.cpp`).
4. Invokes Verilator + the host C++ compiler to produce a shared library.
5. Records all metadata in `external_modules.json` for the simulator build.

The module also provides Rust-facing metadata classes (`FFIPort`, `ExternalFFIModule`) that describe every generated crate.

## Section 1. Exposed Interfaces

### `emit_external_sv_ffis`

```python
def emit_external_sv_ffis(sys, config: dict[str, object], simulator_path: Path, verilator_root: Path) -> List[ExternalFFIModule]:
```

Entry point used during simulator generation. It discovers all `ExternalSV` instances in the system, delegates crate creation to `generate_external_sv_crates`, caches the resulting specs on `sys._external_ffi_specs`, and stores the list in the simulator configuration (`config["external_ffis"]`).

### `generate_external_sv_crates`

```python
def generate_external_sv_crates(modules: Iterable[ExternalSV], simulator_root: Path, verilator_root: Path) -> List[ExternalFFIModule]:
```

Materialises the crates on disk:
  * Removes any stale Verilator workspace.
  * Builds an `ExternalFFIModule` spec for each external block.
  * Writes `Cargo.toml`, `src/lib.rs`, and `src/wrapper.cpp`.
  * Runs Verilator and links the shared library.
  * Emits `external_modules.json` summarising all specs.

Returns the list of populated `ExternalFFIModule` records.

### `FFIPort`

Dataclass capturing the direction, type information, and host language types for a single external port. Used by both Rust and C++ templates.

### `ExternalFFIModule`

Dataclass that tracks all information for a generated crate: crate name, paths, symbol prefix, IO port descriptors, clock/reset flags, and the produced shared library metadata.

These types appear in `__all__`, making them available to other generator components.

## Section 2. Internal Helpers

### `_create_external_spec`

Resolves filenames, allocates unique crate/library names, copies the SystemVerilog source, and populates the `ExternalFFIModule` dataclass. It also calls `_collect_ports` to partition wires into inputs and outputs.

### `_collect_ports` / `_dtype_to_port`

Translate the `ExternalSV.wires` dictionary into `FFIPort` instances. Widths must be ≤ 64 bits—larger ports raise `NotImplementedError`. Signedness automatically selects the appropriate C and Rust scalar types.

### `_generate_cargo_toml`, `_generate_lib_rs`, `_generate_wrapper_cpp`

Emit templated sources for the crate:
  * `Cargo.toml` defines a minimal `libloading` dependency.
  * `src/lib.rs` produces a safe Rust wrapper with dynamic symbol loading, optional clock/reset helpers, and per-port setters/getters.
  * `src/wrapper.cpp` wraps the verilated model with a stable C ABI.

### `_build_verilator_library`

Runs the full native toolchain:
  1. Ensures the `.sv` file is present (`_ensure_sv_source`).
  2. Calls Verilator (`_run_verilator_compile`) into `build/verilated`.
  3. Collects all generated C++ sources (`_gather_source_files`).
  4. Builds the shared library via `_build_compile_command` and `_run_subprocess`.
  5. Writes `.verilator-lib-path` so the Rust wrapper knows where to load the artifact.

### Naming Helpers

`_sanitize_base_name` and `_unique_name` guarantee reproducible yet collision-free crate and library identifiers. Names are normalised with `namify`, falling back to `ext_` prefixes when they would otherwise start with a digit.

## Section 3. Generated Artifacts

- **Crate layout**  
  ```
  verilated_<name>/
    Cargo.toml
    .verilator-lib-path
    rtl/<source>.sv
    src/lib.rs
    src/wrapper.cpp
    build/verilated/...
  ```
- **Shared library**  
  `lib<symbol_prefix>_ffi.{so|dylib|dll}` compiled in the crate root.
- **Manifest**  
  `external_modules.json` (at `simulator_root`) summarises every crate so downstream passes can bind handles and configure Cargo dependencies.

## Section 4. Environment and Failure Modes

- Requires `VERILATOR_ROOT`; absence raises an early error.  
- The C++ toolchain is probed via `CXX` environment variable first, then system-appropriate defaults (clang++ on macOS, c++/g++ on Linux, c++ on other systems); missing toolchains raise `RuntimeError`.  
- Ports wider than 64 bits and missing SystemVerilog sources fail fast.  
- If a system contains no `ExternalSV` modules the Verilator workspace is removed and both `sys._external_ffi_specs` and `config["external_ffis"]` are cleared.

Together these utilities keep external IP mirrored between Verilog and simulator code, ensuring the runtime can load, drive, and sample SystemVerilog blocks with minimal manual wiring.
