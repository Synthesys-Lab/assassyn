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

Entry point used during simulator generation. It discovers all `ExternalSV` usage in the system from two sources:

1. **ExternalSV Module Instances**: Direct module instances in the system
2. **ExternalIntrinsic References**: ExternalSV classes referenced through `ExternalIntrinsic` nodes

For both cases, it generates Verilator FFI crates, caches the resulting specs on `sys._external_ffi_specs` indexed by class/module name, and stores the complete list in the simulator configuration (`config["external_ffis"]`). Name allocation for crates/dynamic libraries is coordinated via `_record_used_name_hints`, while `_generate_class_crates` and `_emit_crate_artifacts` ensure the same build path is reused for both module instances and intrinsic-only classes. This unified approach ensures all external modules use real Verilator-backed FFI regardless of how they are instantiated.

### `generate_external_sv_crates`

```python
def generate_external_sv_crates(modules: Iterable[ExternalSV], simulator_root: Path, verilator_root: Path) -> List[ExternalFFIModule]:
```

Materialises the crates on disk:
  * Removes any stale Verilator workspace.
  * Builds an `ExternalFFIModule` spec for each external block.
  * Delegates to `_emit_crate_artifacts` so file emission and native build logic stay centralised.
  * Emits `external_modules.json` summarising all specs.

Returns the list of populated `ExternalFFIModule` records.

### `FFIPort`

Dataclass capturing the direction, type information, and host language types for a single external port. Used by both Rust and C++ templates.

### `ExternalFFIModule`

Dataclass that tracks all information for a generated crate: crate name, paths, symbol prefix, IO port descriptors, clock/reset flags, parameter overrides, and the produced shared library metadata.

These types appear in `__all__`, making them available to other generator components.

## Section 2. Internal Helpers

### `_create_external_spec`

Resolves filenames, allocates unique crate/library names, copies the SystemVerilog source, and populates the `ExternalFFIModule` dataclass. It also calls `_collect_ports` to partition wires into inputs and outputs. This function works with `ExternalSV` **module instances**.

### `_create_external_spec_from_class`

```python
def _create_external_spec_from_class(external_class: type, verilator_root: Path, used_crate_names: Dict[str, int], used_dynlib_names: Dict[str, int]) -> ExternalFFIModule
```

Creates an `ExternalFFIModule` spec from an `ExternalSV` **class** (rather than an instance). This enables FFI generation for ExternalSV modules used through `ExternalIntrinsic`:

1. Extracts metadata from `external_class.metadata()` to get `'source'` (file path) and `'module_name'` (SystemVerilog module name)
2. Extracts port information from `external_class.port_specs()`
3. Allocates unique crate and dynamic library names
4. Copies the SystemVerilog source to the crate's `rtl/` directory
5. Calls `_collect_ports_from_class` to partition ports into inputs and outputs
6. Copies `metadata["parameters"]` (if any) into the spec so Verilator can apply `-G` overrides
7. Returns a fully populated `ExternalFFIModule` with the class name as `original_module_name`

### `_collect_ports` / `_collect_ports_from_class` / `_dtype_to_port`

**`_collect_ports`**: Translates the `ExternalSV.wires` dictionary (from module instances) into `FFIPort` instances.

**`_collect_ports_from_class`**: Translates the class's `port_specs()` dictionary into `FFIPort` instances. Similar to `_collect_ports` but operates on the class-level port specifications.

**`_dtype_to_port`**: Converts a single port (WireSpec) to an `FFIPort` instance. Widths must be ≤ 64 bits—larger ports raise `NotImplementedError`. Signedness automatically selects the appropriate C and Rust scalar types. Note that WireSpec uses `'in'`/`'out'` for direction, not `'input'`/`'output'`.

### `_generate_cargo_toml`, `_generate_lib_rs`, `_generate_wrapper_cpp`

Emit templated sources for the crate:
  * `Cargo.toml` depends on the shared `sim_runtime` crate (which re-exports `libloading`).
  * `src/lib.rs` produces a safe Rust wrapper with dynamic symbol loading, optional clock/reset helpers, and per-port setters/getters.
  * `src/wrapper.cpp` wraps the verilated model with a stable C ABI.

### `_emit_crate_artifacts`

Writes `Cargo.toml`, `src/lib.rs`, and `src/wrapper.cpp` for a given spec before invoking `_build_verilator_library`. Consolidating these steps keeps both `generate_external_sv_crates` and the class-based generation path in sync.

### `_build_verilator_library`

Runs the full native toolchain:
  1. Ensures the `.sv` file is present (`_ensure_sv_source`).
  2. Calls Verilator (`_run_verilator_compile`) into `build/verilated`, passing any `ExternalSV` parameter overrides via `-G`.
  3. Collects all generated C++ sources (`_gather_source_files`).
  4. Builds the shared library via `_build_compile_command` and `_run_subprocess`.
  5. Writes `.verilator-lib-path` so the Rust wrapper knows where to load the artifact.

### `_write_manifest_file`

Takes a manifest path plus a list of specs and rewrites the JSON summary (including `parameters`) in a single helper. This avoids duplicating the `json.dumps(..., indent=2)` call across the different generation entry points.

### `_record_used_name_hints`

Records the crate and dynamic-library name prefixes that were just generated. By retaining the base names in the `used_*` maps, later specs (e.g. from `ExternalIntrinsic` classes) pick unique suffixes without clobbering the instance-produced crates.

### `_generate_class_crates`

Iterates over the unique `ExternalSV` classes returned from `collect_external_classes`, calling `_create_external_spec_from_class` and `_emit_crate_artifacts` for each. The helper filters out classes lacking a `source` entry so headless stubs do not trigger failing builds.

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
