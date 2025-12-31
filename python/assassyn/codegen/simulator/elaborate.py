"""Elaborate function for Assassyn simulator generator."""

from __future__ import annotations

import os
import shutil
import subprocess
import typing
from pathlib import Path

from .modules import dump_modules
from .simulator import dump_simulator
from .verilator import emit_external_sv_ffis, write_external_ffis_rs

from ...utils import repo_path

if typing.TYPE_CHECKING:
    from ...builder import SysBuilder


def _write_manifest(simulator_path: Path, sys_name: str) -> Path:
    """Write the Cargo manifest for the generated simulator crate."""
    manifest_path = simulator_path / "Cargo.toml"
    runtime_path = Path(repo_path()) / "tools" / "rust-sim-runtime"
    with open(manifest_path, 'w', encoding="utf-8") as cargo:
        cargo.write("[package]\n")
        cargo.write(f'name = "{sys_name}_simulator"\n')
        cargo.write('version = "0.1.0"\n')
        cargo.write('edition = "2021"\n')
        cargo.write('[dependencies]\n')
        cargo.write(f'sim-runtime = {{ path = "{runtime_path}" }}\n')
    return manifest_path


def elaborate_impl(sys, config):
    """Internal implementation of the elaborate function.

    This matches the Rust function in src/backend/simulator/elaborate.rs
    """
    simulator_dirname = (
        config.get('simulator_dirname')
        or config.get('dirname')
        or f"{sys.name}_simulator"
    )
    simulator_path = Path(config.get('path', os.getcwd())) / simulator_dirname
    verilator_root = simulator_path / config.get('verilator_dirname', f"{sys.name}_verilator")

    if simulator_path.exists() and config.get('override_dump', True):
        shutil.rmtree(simulator_path)

    simulator_path.mkdir(parents=True, exist_ok=True)
    (simulator_path / "src").mkdir(exist_ok=True)

    ffi_specs = emit_external_sv_ffis(sys, config, simulator_path, verilator_root)
    write_external_ffis_rs(simulator_path, ffi_specs)

    print(f"Writing simulator code to rust project: {simulator_path}")

    crate_name = config.get('simulator_crate_name') or sys.name
    manifest_path = _write_manifest(simulator_path, crate_name)

    shutil.copy(Path(repo_path()) / "rustfmt.toml", simulator_path / "rustfmt.toml")

    dump_modules(sys, simulator_path / "src" / "modules")

    with open(simulator_path / "src/simulator.rs", 'w', encoding='utf-8') as fd:
        dump_simulator(sys, config, fd)

    shutil.copy(
        Path(__file__).resolve().parent / "template" / "main.rs",
        simulator_path / "src/main.rs"
    )

    return manifest_path


def elaborate(sys, **config):
    """Generate a Rust-based simulator for the given Assassyn system."""

    # pylint: disable=import-outside-toplevel
    from .port_mapper import reset_port_manager
    reset_port_manager()

    manifest_path = elaborate_impl(sys, config)

    try:
        subprocess.run(
            ["cargo", "fmt", "--manifest-path", str(manifest_path)],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        print("Warning: Failed to format code with cargo fmt")

    return manifest_path
