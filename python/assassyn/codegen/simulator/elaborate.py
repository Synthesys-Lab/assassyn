"""Elaborate function for Assassyn simulator generator."""

from __future__ import annotations

import os
import shutil
import subprocess
import typing
from pathlib import Path
from .modules import ElaborateModule
from .simulator import dump_simulator, dump_main
from ...utils import repo_path

if typing.TYPE_CHECKING:
    from ...builder import SysBuilder


def dump_modules(sys: SysBuilder, fd):
    """Generate the modules.rs file.

    This matches the Rust function in src/backend/simulator/elaborate.rs
    """
    # Add imports
    fd.write("""
use sim_runtime::*;
use super::simulator::Simulator;
use std::collections::VecDeque;
use sim_runtime::num_bigint::{BigInt, BigUint};
use sim_runtime::libloading::{Library, Symbol};
use std::ffi::{CString, c_char, c_float, c_longlong, c_void};
use std::sync::Arc;
    """)

    # Generate each module's implementation
    dict_modules_callback = {}
    em = ElaborateModule(sys)
    for module in sys.modules[:] + sys.downstreams[:]:
        dict_modules_callback = em.visit_module_for_callback(module)
    required_keys = ["memory", "store", "MemUser_rdata"]
    if all(dict_modules_callback.get(k) is not None for k in required_keys):
        fd.write(f"""
    extern "C" fn rust_callback(req: *mut Request, ctx: *mut c_void) {{
        unsafe {{
            let req = &*req;
            let sim: &mut Simulator = &mut *(ctx as *mut Simulator);
            let cycles = (req.depart - req.arrive) as usize;
            let stamp = sim.request_stamp_map_table
                .remove(&req.addr)
                .unwrap_or_else(|| sim.stamp);;
            sim.{dict_modules_callback.get("MemUser_rdata")}.push.push(FIFOPush::new(
                stamp + 100 * cycles,
                sim.{dict_modules_callback.
                     get("store")}.payload[req.addr as usize].clone().try_into().unwrap(),
                "{dict_modules_callback.get("memory")}",
            ));
        }}
    }}""")
    for module in sys.modules[:] + sys.downstreams[:]:
        # Then, second time dump for real visit modules
        module_code = em.visit_module(module)
        fd.write(module_code)

    return True


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

    # Create rustfmt.toml if available
    rustfmt_src = None
    rustfmt_candidates = [simulator_path / "rustfmt.toml", Path(repo_path()) / "rustfmt.toml"]

    for candidate in rustfmt_candidates:
        if candidate.exists():
            rustfmt_src = candidate
            break

    if rustfmt_src:
        shutil.copy(rustfmt_src, simulator_path / "rustfmt.toml")

    # Generate modules.rs
    with open(simulator_path / "src/modules.rs", 'w', encoding="utf-8") as fd:
        dump_modules(sys, fd)

    # Generate simulator.rs
    with open(simulator_path / "src/simulator.rs", 'w', encoding='utf-8') as fd:
        dump_simulator(sys, config, fd)

    # Generate main.rs
    with open(simulator_path / "src/main.rs", 'w', encoding='utf-8') as fd:
        dump_main(fd)

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
