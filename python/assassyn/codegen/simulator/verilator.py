"""Helpers for generating and wiring Verilator FFI crates for ExternalSV."""

from __future__ import annotations

import json
import os
import platform
import shlex
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, Iterable, List, Optional

from ...ir.dtype import DType
from ...ir.module.external import ExternalSV
from ...ir.module.module import Wire
from ...utils import namify, repo_path
from .utils import camelize


_C_INT_TYPES_UNSIGNED = {8: "uint8_t", 16: "uint16_t", 32: "uint32_t", 64: "uint64_t"}
_C_INT_TYPES_SIGNED = {8: "int8_t", 16: "int16_t", 32: "int32_t", 64: "int64_t"}
_RUST_INT_TYPES_UNSIGNED = {8: "u8", 16: "u16", 32: "u32", 64: "u64"}
_RUST_INT_TYPES_SIGNED = {8: "i8", 16: "i16", 32: "i32", 64: "i64"}


@dataclass
class FFIPort:
    """Description of a single ExternalSV port used for FFI generation."""

    name: str
    direction: str
    dtype: DType
    bits: int
    signed: bool
    c_type: str
    rust_type: str


@dataclass
class ExternalFFIModule:  # pylint: disable=too-many-instance-attributes
    """Artifacts emitted for a single ExternalSV module."""

    crate_name: str
    crate_path: Path
    symbol_prefix: str
    dynamic_lib_name: str
    top_module: str
    sv_filename: str
    sv_rel_path: str
    inputs: List[FFIPort] = field(default_factory=list)
    outputs: List[FFIPort] = field(default_factory=list)
    has_clock: bool = False
    has_reset: bool = False
    original_module_name: str = ""
    struct_name: str = ""
    definitions: Dict[str, str] = field(default_factory=dict)
    lib_filename: str = ""
    lib_path: Optional[Path] = None


def _storage_width(bits: int) -> int:
    if bits <= 8:
        return 8
    if bits <= 16:
        return 16
    if bits <= 32:
        return 32
    if bits <= 64:
        return 64
    raise NotImplementedError(
        f"ExternalSV wires wider than 64 bits are not yet supported (requested {bits} bits)"
    )


def _dtype_to_port(name: str, wire: Wire) -> FFIPort:
    dtype = wire.dtype
    bits = getattr(dtype, "bits", None)
    if bits is None:
        raise ValueError(f"Wire '{name}' lacks a bit-width definition")
    storage_bits = _storage_width(bits)
    signed = dtype.is_signed()
    if signed:
        c_type = _C_INT_TYPES_SIGNED[storage_bits]
        rust_type = _RUST_INT_TYPES_SIGNED[storage_bits]
    else:
        c_type = _C_INT_TYPES_UNSIGNED[storage_bits]
        rust_type = _RUST_INT_TYPES_UNSIGNED[storage_bits]
    return FFIPort(
        name=namify(name),
        direction=wire.direction or "input",
        dtype=dtype,
        bits=bits,
        signed=signed,
        c_type=c_type,
        rust_type=rust_type,
    )


def _ensure_repo_local_path(file_path: str) -> Path:
    src = Path(file_path)
    if src.is_absolute():
        return src
    return Path(repo_path()) / src


def _write_file(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _dynamic_lib_suffix() -> str:
    system = platform.system().lower()
    if system == "windows":
        return ".dll"
    if system == "darwin":
        return ".dylib"
    return ".so"


def _compiler_command() -> List[str]:
    """Detect and return the appropriate C++ compiler command."""
    # First, check if CXX environment variable is set
    compiler_env = os.environ.get("CXX")
    if compiler_env:
        tokens = shlex.split(compiler_env)
        if tokens:
            return tokens
    # Try to detect the system's default C++ compiler more intelligently
    # Check for common compiler environment variables
    for env_var in ["CXX", "CC"]:
        if env_var in os.environ:
            compiler_path = os.environ[env_var]
            if compiler_path and shutil.which(compiler_path):
                return [compiler_path]
    # Fallback to common C++ compilers, but try to be more system-appropriate
    candidates = []
    # On macOS, prefer clang++ if available (it's the default)
    if sys.platform == "darwin":
        candidates = ["clang++", "g++", "c++"]
    # On Linux, prefer c++ (generic) then g++, then clang++
    elif sys.platform.startswith("linux"):
        candidates = ["c++", "g++", "clang++"]
    # On other systems, use a generic order
    else:
        candidates = ["c++", "g++", "clang++"]
    for candidate in candidates:
        path = shutil.which(candidate)
        if path:
            return [path]
    raise RuntimeError(
        "Unable to locate a C++ compiler. Please set the CXX environment variable "
        "or install a C++ compiler (g++, clang++, or c++)."
    )


def _run_subprocess(cmd: List[str], cwd: Path | None = None) -> None:
    subprocess.run(cmd, check=True, cwd=cwd, env=os.environ.copy())


def _ensure_sv_source(crate: ExternalFFIModule) -> Path:
    """Return the absolute path to the SystemVerilog source and ensure it exists."""
    sv_source = crate.crate_path / crate.sv_rel_path
    if not sv_source.exists():
        raise FileNotFoundError(f"SystemVerilog source not found: {sv_source}")
    return sv_source


def _prepare_build_directory(crate: ExternalFFIModule) -> Path:
    """Reset and create the Verilator build directory."""
    build_root = crate.crate_path / "build"
    obj_dir = build_root / "verilated"
    shutil.rmtree(build_root, ignore_errors=True)
    obj_dir.mkdir(parents=True, exist_ok=True)
    return obj_dir


def _run_verilator_compile(crate: ExternalFFIModule, sv_source: Path, obj_dir: Path) -> None:
    """Invoke Verilator to generate the C++ model."""
    verilator_exe = os.environ.get("ASSASSYN_VERILATOR", "verilator")
    verilator_cmd = [
        verilator_exe,
        "--cc",
        str(sv_source),
        "--top-module",
        crate.top_module,
        "-O3",
        "--Mdir",
        str(obj_dir),
    ]
    _run_subprocess(verilator_cmd)


def _resolve_verilator_paths() -> tuple[Path, Path]:
    """Locate the Verilator include directories."""
    verilator_root = os.environ.get("VERILATOR_ROOT")
    if not verilator_root:
        raise EnvironmentError(
            "VERILATOR_ROOT is not set. Please run 'source setup.sh' before "
            "generating external FFIs."
        )
    include_dir = Path(verilator_root) / "include"
    if not include_dir.exists():
        raise FileNotFoundError(f"Verilator include directory not found: {include_dir}")
    return include_dir, include_dir / "vltstd"


def _gather_source_files(
    crate: ExternalFFIModule,
    obj_dir: Path,
    include_dir: Path,
) -> List[Path]:
    """Collect all C++ sources required to build the shared library."""
    cpp_class = f"V{crate.top_module}"
    aggregated = obj_dir / f"{cpp_class}__ALL.cpp"

    source_files: List[Path] = []
    if aggregated.exists():
        source_files.append(aggregated)
    else:
        for path in sorted(obj_dir.glob("*.cpp")):
            if path.name.endswith("__ALL.cpp"):
                continue
            source_files.append(path)

    wrapper_src = crate.crate_path / "src" / "wrapper.cpp"
    if not wrapper_src.exists():
        raise FileNotFoundError(f"Wrapper source not found: {wrapper_src}")
    source_files.append(wrapper_src)

    runtime_sources = [include_dir / "verilated.cpp"]
    for extra in ("verilated_threads.cpp", "verilated_dpi.cpp"):
        extra_path = include_dir / extra
        if extra_path.exists():
            runtime_sources.append(extra_path)
    source_files.extend(runtime_sources)
    return source_files


def _build_compile_command(
    crate: ExternalFFIModule,
    source_files: List[Path],
    include_dir: Path,
    vltstd_dir: Path,
    obj_dir: Path,
) -> tuple[List[str], str, Path]:
    """Construct the compiler command for the shared library."""
    compiler = _compiler_command()
    compile_cmd = compiler + [
        "-std=c++17",
        "-shared",
        "-fPIC",
        "-O3",
    ]
    for include in (include_dir, vltstd_dir, obj_dir):
        compile_cmd.extend(["-I", str(include)])
    compile_cmd.extend(str(src) for src in source_files)

    lib_filename = f"lib{crate.dynamic_lib_name}{_dynamic_lib_suffix()}"
    lib_path = crate.crate_path / lib_filename
    compile_cmd.extend(["-o", str(lib_path)])
    return compile_cmd, lib_filename, lib_path


def _unique_name(base: str, registry: Dict[str, int]) -> str:
    """Return a unique name derived from base and update the registry."""
    count = registry.get(base, 0)
    registry[base] = count + 1
    return base if count == 0 else f"{base}_{count + 1}"


def _sanitize_base_name(top_module: str, fallback: str) -> str:
    """Normalize the base name used for crate generation."""
    base = namify(top_module) or namify(fallback)
    if not base:
        base = "external"
    if base[0].isdigit():
        base = f"ext_{base}"
    return base


def _collect_ports(module: ExternalSV) -> tuple[List[FFIPort], List[FFIPort]]:
    """Split module wires into input and output ports for FFI generation."""
    ports_in: List[FFIPort] = []
    ports_out: List[FFIPort] = []
    for name, wire in module.wires.items():
        port = _dtype_to_port(name, wire)
        if port.direction == "output":
            ports_out.append(port)
        else:
            ports_in.append(port)
    return ports_in, ports_out


def _create_external_spec(
    module: ExternalSV,
    verilator_root: Path,
    used_crate_names: Dict[str, int],
    used_dynlib_names: Dict[str, int],
) -> ExternalFFIModule:
    """Create an ExternalFFIModule description and prepare the crate directory."""
    top_module = module.external_module_name
    if not top_module:
        raise ValueError("ExternalSV module must specify 'module_name' to drive Verilator")

    base_name = _sanitize_base_name(top_module, module.name)
    crate_name = _unique_name(f"verilated_{base_name}", used_crate_names)
    symbol_prefix = namify(crate_name)
    dynlib_base = f"{symbol_prefix}_ffi"
    dynamic_lib_name = namify(_unique_name(dynlib_base, used_dynlib_names))

    crate_path = verilator_root / crate_name
    crate_path.mkdir(parents=True, exist_ok=True)
    (crate_path / "src").mkdir(exist_ok=True)
    (crate_path / "rtl").mkdir(exist_ok=True)

    src_sv_path = _ensure_repo_local_path(module.file_path)
    if not src_sv_path.exists():
        raise FileNotFoundError(f"ExternalSV file not found: {src_sv_path}")
    dst_sv_path = crate_path / "rtl" / src_sv_path.name
    shutil.copy(src_sv_path, dst_sv_path)

    ports_in, ports_out = _collect_ports(module)

    return ExternalFFIModule(
        crate_name=crate_name,
        crate_path=crate_path,
        symbol_prefix=symbol_prefix,
        dynamic_lib_name=dynamic_lib_name,
        top_module=top_module,
        sv_filename=src_sv_path.name,
        sv_rel_path=os.path.join("rtl", src_sv_path.name),
        inputs=ports_in,
        outputs=ports_out,
        has_clock=getattr(module, "has_clock", False),
        has_reset=getattr(module, "has_reset", False),
        original_module_name=module.name,
    )


def _spec_manifest_entry(spec: ExternalFFIModule, simulator_root: Path) -> Dict[str, object]:
    """Return a manifest entry describing a compiled ExternalFFIModule."""
    return {
        "crate": spec.crate_name,
        "dynamic_lib": spec.dynamic_lib_name,
        "top_module": spec.top_module,
        "sv": spec.sv_filename,
        "crate_dir": os.path.relpath(spec.crate_path, simulator_root),
        "struct_name": spec.struct_name,
        "has_clock": spec.has_clock,
        "has_reset": spec.has_reset,
        "lib_filename": spec.lib_filename,
        "lib_path": str(spec.lib_path) if spec.lib_path else "",
        "inputs": [
            {
                "name": port.name,
                "bits": port.bits,
                "signed": port.signed,
                "rust_type": port.rust_type,
                "c_type": port.c_type,
            }
            for port in spec.inputs
        ],
        "outputs": [
            {
                "name": port.name,
                "bits": port.bits,
                "signed": port.signed,
                "rust_type": port.rust_type,
                "c_type": port.c_type,
            }
            for port in spec.outputs
        ],
        "original_module_name": spec.original_module_name,
    }


def _generate_cargo_toml(crate: ExternalFFIModule) -> str:
    return f"""[package]
name = "{crate.crate_name}"
version = "0.1.0"
edition = "2021"
[dependencies]
libloading = "0.8"
"""


def _generate_lib_rs(crate: ExternalFFIModule) -> str:  # pylint: disable=too-many-branches
    # pylint: disable=too-many-locals, too-many-statements
    struct_name = camelize(crate.symbol_prefix) or "ExternalModule"
    struct_name = struct_name[0].upper() + struct_name[1:]
    crate.struct_name = struct_name
    prefix = crate.symbol_prefix

    lines = [
        "#![allow(dead_code)]",
        "use libloading::Library;",
        "use std::path::{Path, PathBuf};",
        "use std::ptr::NonNull;",
        "",
        "#[repr(C)]",
        "#[allow(non_camel_case_types)]",
        "pub struct ModuleHandle { _private: [u8; 0] }",
        "",
        (
            "const LIB_PATH: &str = include_str!(concat!("
            "env!(\"CARGO_MANIFEST_DIR\"), \"/.verilator-lib-path\"));"
        ),
        "",
        "fn lib_path() -> PathBuf {",
        "    PathBuf::from(LIB_PATH.trim())",
        "}",
        "",
        "fn load_library<P: AsRef<Path>>(path: P) -> Library {",
        "    let path = path.as_ref();",
        (
            "    unsafe { Library::new(path) }."
            "unwrap_or_else(|err| panic!(\"failed to load Verilator library '"
            f"{prefix}': {{err}} ({{}})\", path.display()))"
        ),
        "}",
        "",
        "unsafe fn load_symbol<T: Copy>(lib: &Library, symbol: &[u8], name: &str) -> T {",
        (
            "    *lib.get::<T>(symbol).unwrap_or_else(|err| "
            "panic!(\"failed to load symbol {name}: {err}\"))"
        ),
        "}",
        "",
        f"pub struct {struct_name} {{",
        "    lib: Library,",
        "    handle: NonNull<ModuleHandle>,",
        "    free_fn: unsafe extern \"C\" fn(*mut ModuleHandle),",
        "    eval_fn: unsafe extern \"C\" fn(*mut ModuleHandle),",
    ]

    if crate.has_clock:
        lines.append("    set_clk_fn: unsafe extern \"C\" fn(*mut ModuleHandle, u8),")
        lines.append("    clk_state: u8,")
    if crate.has_reset:
        lines.append("    set_rst_fn: unsafe extern \"C\" fn(*mut ModuleHandle, u8),")
        lines.append("    rst_state: u8,")
    for port in crate.inputs:
        lines.append(
            (
                f"    set_{port.name}_fn: unsafe extern \"C\" fn(*mut ModuleHandle, "
                f"{port.rust_type}),"
            )
        )
    for port in crate.outputs:
        lines.append(
            (
                f"    get_{port.name}_fn: unsafe extern \"C\" fn(*mut ModuleHandle) -> "
                f"{port.rust_type},"
            )
        )
    lines.append("}")
    lines.append("")

    impl_lines = [
        f"impl {struct_name} {{",
        "    pub fn new() -> Self {",
        "        let path = lib_path();",
        "        Self::new_from_path(path)",
        "    }",
        "",
        "    pub fn new_from_path<P: AsRef<Path>>(path: P) -> Self {",
        "        let lib = load_library(path);",
        "        unsafe {",
        (
            "            let new_fn: unsafe extern \"C\" fn() -> *mut ModuleHandle = "
            f"load_symbol(&lib, b\"{prefix}_new\", \"{prefix}_new\");"
        ),
        (
            "            let free_fn: unsafe extern \"C\" fn(*mut ModuleHandle) = "
            f"load_symbol(&lib, b\"{prefix}_free\", \"{prefix}_free\");"
        ),
        (
            "            let eval_fn: unsafe extern \"C\" fn(*mut ModuleHandle) = "
            f"load_symbol(&lib, b\"{prefix}_eval\", \"{prefix}_eval\");"
        ),
    ]

    if crate.has_clock:
        impl_lines.append(
            (
                "            let set_clk_fn: unsafe extern \"C\" fn(*mut ModuleHandle, u8) = "
                f"load_symbol(&lib, b\"{prefix}_set_clk\", \"{prefix}_set_clk\");"
            )
        )
    if crate.has_reset:
        impl_lines.append(
            (
                "            let set_rst_fn: unsafe extern \"C\" fn(*mut ModuleHandle, u8) = "
                f"load_symbol(&lib, b\"{prefix}_set_rst\", \"{prefix}_set_rst\");"
            )
        )
    for port in crate.inputs:
        impl_lines.append(
            (
                f"            let set_{port.name}_fn: unsafe extern \"C\" fn(*mut ModuleHandle, "
                f"{port.rust_type}) = load_symbol(&lib, b\"{prefix}_set_{port.name}\", "
                f"\"{prefix}_set_{port.name}\");"
            )
        )
    for port in crate.outputs:
        impl_lines.append(
            (
                f"            let get_{port.name}_fn: unsafe extern \"C\" fn(*mut ModuleHandle) -> "
                f"{port.rust_type} = load_symbol(&lib, b\"{prefix}_get_{port.name}\", "
                f"\"{prefix}_get_{port.name}\");"
            )
        )
    impl_lines.append(
        (
            "            let handle = NonNull::new(new_fn())."
            f"unwrap_or_else(|| panic!(\"{prefix}_new returned null\"));"
        )
    )
    impl_lines.append("            let mut instance = Self {")
    impl_lines.append("                lib,")
    impl_lines.append("                handle,")
    impl_lines.append("                free_fn,")
    impl_lines.append("                eval_fn,")
    if crate.has_clock:
        impl_lines.append("                set_clk_fn,")
        impl_lines.append("                clk_state: 0,")
    if crate.has_reset:
        impl_lines.append("                set_rst_fn,")
        impl_lines.append("                rst_state: 0,")
    for port in crate.inputs:
        impl_lines.append(f"                set_{port.name}_fn,")
    for port in crate.outputs:
        impl_lines.append(f"                get_{port.name}_fn,")
    impl_lines.append("            };")
    if crate.has_clock:
        impl_lines.append("            set_clk_fn(instance.handle.as_ptr(), 0);")
    if crate.has_reset:
        impl_lines.append("            set_rst_fn(instance.handle.as_ptr(), 0);")
    impl_lines.append("            instance")
    impl_lines.append("        }")
    impl_lines.append("    }")
    impl_lines.append("")
    impl_lines.append(
        "    pub fn eval(&mut self) { unsafe { (self.eval_fn)(self.handle.as_ptr()) } }"
    )
    impl_lines.append("")

    if crate.has_clock:
        impl_lines.extend(
            [
                "    pub fn set_clock(&mut self, value: bool) {",
                "        let value = value as u8;",
                "        unsafe { (self.set_clk_fn)(self.handle.as_ptr(), value) };",
                "        self.clk_state = value;",
                "    }",
                "",
                "    pub fn clock_tick(&mut self) {",
                "        self.set_clock(false);",
                "        self.eval();",
                "        self.set_clock(true);",
                "        self.eval();",
                "    }",
                "",
            ]
        )
    if crate.has_reset:
        impl_lines.extend(
            [
                "    pub fn set_reset(&mut self, value: bool) {",
                "        let value = value as u8;",
                "        unsafe { (self.set_rst_fn)(self.handle.as_ptr(), value) };",
                "        self.rst_state = value;",
                "    }",
                "",
            ]
        )
        if crate.has_clock:
            impl_lines.extend(
                [
                    "    pub fn apply_reset(&mut self, cycles: usize) {",
                    "        self.set_reset(true);",
                    "        for _ in 0..cycles.max(1) {",
                    "            self.clock_tick();",
                    "        }",
                    "        self.set_reset(false);",
                    "        self.clock_tick();",
                    "    }",
                    "",
                ]
            )
        else:
            impl_lines.extend(
                [
                    "    pub fn apply_reset(&mut self, cycles: usize) {",
                    "        let _ = cycles;",
                    "        self.set_reset(true);",
                    "        self.eval();",
                    "        self.set_reset(false);",
                    "        self.eval();",
                    "    }",
                    "",
                ]
            )

    for port in crate.inputs:
        impl_lines.extend(
            [
                f"    pub fn set_{port.name}(&mut self, value: {port.rust_type}) {{",
                f"        unsafe {{ (self.set_{port.name}_fn)(self.handle.as_ptr(), value) }};",
                "    }",
                "",
            ]
        )
    for port in crate.outputs:
        impl_lines.extend(
            [
                f"    pub fn get_{port.name}(&mut self) -> {port.rust_type} {{",
                f"        unsafe {{ (self.get_{port.name}_fn)(self.handle.as_ptr()) }}",
                "    }",
                "",
            ]
        )
    impl_lines.append("}")
    lines.extend(impl_lines)
    lines.append("")
    lines.append(f"impl Drop for {struct_name} {{")
    lines.append(
        "    fn drop(&mut self) { unsafe { (self.free_fn)(self.handle.as_ptr()) } }"
    )
    lines.append("}")

    return "\n".join(lines)


def _generate_wrapper_cpp(crate: ExternalFFIModule) -> str:
    cpp_class = f"V{crate.top_module}"
    prefix = crate.symbol_prefix
    lines = [
        f"#include \"{cpp_class}.h\"",
        "#include \"verilated.h\"",
        "#include <cstdint>",
        "",
        "double sc_time_stamp() { return 0.0; }",
        "",
        "extern \"C\" {",
        "",
        f"using ModuleHandle = {cpp_class};",
        "",
        f"ModuleHandle* {prefix}_new() {{",
        "    static bool inited = false;",
        "    if (!inited) { Verilated::debug(0); inited = true; }",
        "    return new ModuleHandle();",
        "}",
        "",
        f"void {prefix}_free(ModuleHandle* handle) {{ delete handle; }}",
        "",
        f"void {prefix}_eval(ModuleHandle* handle) {{ handle->eval(); }}",
    ]
    if crate.has_clock:
        lines.extend(
            [
                f"void {prefix}_set_clk(ModuleHandle* handle, uint8_t value) {{",
                "    handle->clk = static_cast<uint8_t>(value & 0x1U);",
                "}",
            ]
        )
    if crate.has_reset:
        lines.extend(
            [
                f"void {prefix}_set_rst(ModuleHandle* handle, uint8_t value) {{",
                "    handle->rst = static_cast<uint8_t>(value & 0x1U);",
                "}",
            ]
        )
    for port in crate.inputs:
        lines.extend(
            [
                f"void {prefix}_set_{port.name}(ModuleHandle* handle, {port.c_type} value) {{",
                f"    handle->{port.name} = static_cast<{port.c_type}>(value);",
                "}",
            ]
        )
    for port in crate.outputs:
        lines.extend(
            [
                f"{port.c_type} {prefix}_get_{port.name}(ModuleHandle* handle) {{",
                f"    return static_cast<{port.c_type}>(handle->{port.name});",
                "}",
            ]
        )
    lines.append("}")
    return "\n".join(lines) + "\n"


def _build_verilator_library(crate: ExternalFFIModule) -> Path:
    """Compile the Verilator-generated model and wrapper into a shared library."""

    sv_source = _ensure_sv_source(crate)
    obj_dir = _prepare_build_directory(crate)
    _run_verilator_compile(crate, sv_source, obj_dir)
    include_dir, vltstd_dir = _resolve_verilator_paths()
    source_files = _gather_source_files(crate, obj_dir, include_dir)
    compile_cmd, lib_filename, lib_path = _build_compile_command(
        crate,
        source_files,
        include_dir,
        vltstd_dir,
        obj_dir,
    )
    _run_subprocess(compile_cmd)

    crate.lib_filename = lib_filename
    crate.lib_path = lib_path

    _write_file(crate.crate_path / ".verilator-lib-path", str(lib_path.resolve()))
    return lib_path


def generate_external_sv_crates(
    modules: Iterable[ExternalSV],
    simulator_root: Path,
    verilator_root: Path,
) -> List[ExternalFFIModule]:
    """Generate Verilator FFI crates for the provided ExternalSV modules."""

    specs: List[ExternalFFIModule] = []
    used_crate_names: Dict[str, int] = {}
    used_dynlib_names: Dict[str, int] = {}

    shutil.rmtree(verilator_root, ignore_errors=True)
    verilator_root.mkdir(parents=True, exist_ok=True)

    for module in modules:
        if not getattr(module, "file_path", None):
            continue

        spec = _create_external_spec(module, verilator_root, used_crate_names, used_dynlib_names)
        cargo_toml = _generate_cargo_toml(spec)
        lib_rs = _generate_lib_rs(spec)
        wrapper_cpp = _generate_wrapper_cpp(spec)

        _write_file(spec.crate_path / "Cargo.toml", cargo_toml)
        _write_file(spec.crate_path / "src/lib.rs", lib_rs)
        _write_file(spec.crate_path / "src/wrapper.cpp", wrapper_cpp)

        _build_verilator_library(spec)
        specs.append(spec)

    if specs:
        manifest = {"modules": [_spec_manifest_entry(spec, simulator_root) for spec in specs]}
        _write_file(simulator_root / "external_modules.json", json.dumps(manifest, indent=2))

    return specs


def emit_external_sv_ffis(
    sys_module,
    config: dict[str, object],
    simulator_path: Path,
    verilator_root: Path,
) -> List[ExternalFFIModule]:
    """Generate Verilator crates for ExternalSV modules and record their specs."""

    modules = [
        module
        for module in getattr(sys_module, "modules", []) + getattr(sys_module, "downstreams", [])
        if isinstance(module, ExternalSV)
    ]

    if not modules:
        shutil.rmtree(verilator_root, ignore_errors=True)
        sys_module._external_ffi_specs = {}  # pylint: disable=protected-access
        config["external_ffis"] = []
        return []

    ffi_specs = generate_external_sv_crates(modules, simulator_path, verilator_root)
    sys_module._external_ffi_specs = {  # pylint: disable=protected-access
        spec.original_module_name: spec for spec in ffi_specs
    }
    config["external_ffis"] = ffi_specs
    return ffi_specs


__all__ = [
    "emit_external_sv_ffis",
    "generate_external_sv_crates",
    "ExternalFFIModule",
    "FFIPort",
]
