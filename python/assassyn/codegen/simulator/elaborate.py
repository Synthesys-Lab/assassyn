"""Elaborate function for Assassyn simulator generator."""

from __future__ import annotations

import os
import shutil
import subprocess
import typing
from pathlib import Path
from .modules import dump_modules
from .simulator import dump_simulator
from ...utils import repo_path

if typing.TYPE_CHECKING:
    from ...builder import SysBuilder


def elaborate_impl(sys, config):
    """Internal implementation of the elaborate function.

    This matches the Rust function in src/backend/simulator/elaborate.rs
    """
    # Create and clean the simulator directory
    simulator_name = config.get('dirname', f"{sys.name}_simulator")
    simulator_path = Path(config.get('path', os.getcwd())) / simulator_name

    # Clean directory if it exists and override is enabled
    if simulator_path.exists() and config.get('override_dump', True):
        shutil.rmtree(simulator_path)

    # Create directories
    simulator_path.mkdir(parents=True, exist_ok=True)
    (simulator_path / "src").mkdir(exist_ok=True)

    print(f"Writing simulator code to rust project: {simulator_path}")

    # Create Cargo.toml
    manifest_path = simulator_path / "Cargo.toml"
    runtime_path = Path(repo_path()) / "tools" / "rust-sim-runtime"
    with open(manifest_path, 'w', encoding="utf-8") as cargo:
        cargo.write("[package]\n")
        cargo.write(f'name = "{sys.name}_simulator"\n')
        cargo.write('version = "0.1.0"\n')
        cargo.write('edition = "2021"\n')
        cargo.write('[dependencies]\n')
        cargo.write(f'sim-runtime = {{ path = "{runtime_path}" }}\n')

    # Create rustfmt for the generated project
    shutil.copy(Path(repo_path()) / "rustfmt.toml", simulator_path / "rustfmt.toml")

    # Generate modules.rs
    with open(simulator_path / "src/modules.rs", 'w', encoding="utf-8") as fd:
        dump_modules(sys, fd)

    # Generate simulator.rs
    with open(simulator_path / "src/simulator.rs", 'w', encoding='utf-8') as fd:
        dump_simulator(sys, config, fd)

    # Generate main.rs
    template_main = Path(__file__).resolve().parent / "template" / "main.rs"
    shutil.copy(template_main, simulator_path / "src/main.rs")

    return manifest_path


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

    # Generate the simulator
    manifest_path = elaborate_impl(sys, config)

    # Format the code if cargo fmt is available
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
