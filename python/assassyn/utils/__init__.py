"""Utility functions and decorators for Assassyn."""

from __future__ import annotations

# Standard library imports
import os
import subprocess
import sys
import re
import glob
import hashlib
import json
# Local imports
from .enforce_type import enforce_type, validate_arguments, check_type

# Cache coordination data between elaborate() and build_simulator()
CACHE_PENDING: tuple[str, str, str] | None = None

def identifierize(obj):
    '''The helper function to get the identifier of the given object. You can change `id_slice`
    to tune the length of the identifier. The default is slice(-6:-1).'''
    # pylint: disable=import-outside-toplevel
    from ..builder import Singleton
    return hex(id(obj))[Singleton.id_slice]

def unwrap_operand(node):
    """Unwrap the operand from the node.

    This is a helper function to get the operand from the node.
    """
    # pylint: disable=import-outside-toplevel
    from ..ir.expr import Operand
    if isinstance(node, Operand):
        return node.value
    return node

PATH_CACHE: str | None = None
VERILATOR_CACHE: str | None = None

def repo_path() -> str:
    """Get the path to assassyn repository.
    
    Returns:
        str: Path to the repository root
        
    Raises:
        EnvironmentError: If ASSASSYN_HOME environment variable is not set
    """
    # pylint: disable=global-statement
    global PATH_CACHE
    if PATH_CACHE is None:
        try:
            PATH_CACHE = os.environ['ASSASSYN_HOME']
        except KeyError as e:
            raise EnvironmentError(
                "ASSASSYN_HOME environment variable not set. "
                "Please run 'source setup.sh' first."
            ) from e
    return PATH_CACHE

def package_path() -> str:
    """Get the path to this python package."""
    return os.path.join(repo_path(), 'python', 'assassyn')

def _cmd_wrapper(cmd):
    env = os.environ.copy()
    env.pop('RUSTC_WRAPPER', None)  # sccache fails under some sandboxed runners
    return subprocess.check_output(cmd, env=env).decode('utf-8')

def patch_fifo(file_path):
    """
    Replaces all occurrences of 'fifo_n #(' with 'fifo #(' in the Top.sv
    """
    if not os.path.isfile(file_path):
        return

    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    pattern = re.compile(r'fifo_\d+\s*#\s*\(')
    replacement = 'fifo #('
    new_content, num_replacements = pattern.subn(replacement, content)
    if num_replacements > 0:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(new_content)


def get_simulator_binary_path(manifest_path):
    '''Get the path to the compiled simulator binary.

    Args:
        manifest_path: Path to Cargo.toml

    Returns:
        str: Path to the compiled binary
    '''
    # pylint: disable=import-outside-toplevel
    try:
        # Python 3.11+ has tomllib in the standard library
        import tomllib
    except ImportError:
        # Fallback to toml package for Python < 3.11
        import toml as tomllib  # type: ignore

    manifest_dir = os.path.dirname(os.path.abspath(manifest_path))

    # Parse Cargo.toml to get package name
    with open(manifest_path, 'rb') as f:
        cargo_toml = tomllib.load(f)

    package_name = cargo_toml['package']['name']

    # Check if CARGO_TARGET_DIR is set (used by the build system)
    cargo_target_dir = os.environ.get('CARGO_TARGET_DIR')
    if cargo_target_dir:
        binary_path = os.path.join(cargo_target_dir, 'release', package_name)
    else:
        binary_path = os.path.join(manifest_dir, 'target', 'release', package_name)

    return binary_path


def build_simulator(manifest_path, offline=False):
    '''Build the simulator binary using cargo build.

    Args:
        manifest_path: Path to Cargo.toml
        offline: Whether to use offline mode

    Returns:
        str: Path to the compiled binary
    '''
    # If it's already a binary (from cache), just return it
    if os.path.exists(manifest_path) and not str(manifest_path).endswith('.toml'):
        print(f"[Cache] Using cached binary: {manifest_path}")
        return manifest_path

    def _build(off):
        cmd = ['cargo', 'build', '--release', '--manifest-path', manifest_path]
        if off:
            cmd += ['--offline']
        print(cmd)
        _cmd_wrapper(cmd)

    try:
        _build(offline)
    except subprocess.CalledProcessError as err:
        if offline:
            raise
        try:
            _build(True)
        except subprocess.CalledProcessError as retry_err:
            raise err from retry_err

    binary_path = get_simulator_binary_path(manifest_path)

    # Save cache if elaborate() set up cache info
    # pylint: disable=global-statement
    global CACHE_PENDING
    cache_data = CACHE_PENDING
    if cache_data is not None:
        source_dir, cache_key, verilog_path = cache_data
        save_build_cache(source_dir, cache_key, binary_path, verilog_path)
        print("[Cache Saved] Build cached for future use")
        CACHE_PENDING = None

    return binary_path


def run_simulator(manifest_path=None, offline=False, release=True, binary_path=None):
    '''The helper function to run the simulator.

    Args:
        manifest_path: Path to Cargo.toml (used if binary_path is None)
        offline: Whether to use offline mode
        release: Whether to use release mode
        binary_path: Path to compiled binary (if provided, run directly)

    Returns:
        str: Output from the simulator
    '''
    if binary_path is not None:
        # Run the binary directly
        print([binary_path])
        return _cmd_wrapper([binary_path])

    # Fall back to cargo run
    def _run(off):
        cmd = ['cargo', 'run', '--manifest-path', manifest_path]
        if off:
            cmd += ['--offline']
        if release:
            cmd += ['--release']
        print(cmd)
        return _cmd_wrapper(cmd)

    try:
        return _run(offline)
    except subprocess.CalledProcessError as err:
        if offline:
            raise
        try:
            return _run(True)
        except subprocess.CalledProcessError as retry_err:
            raise err from retry_err

def run_verilator(path):
    '''The helper function to run the verilator'''
    restore = os.getcwd()
    os.chdir(path)
    cmd_design = ['python', 'design.py']
    subprocess.check_output(cmd_design)
    patch_fifo("sv/hw/Top.sv")
    cmd_tb = ['python', 'tb.py']
    res = _cmd_wrapper(cmd_tb)
    # Filter infrastructure logs (e.g., INFO: Running command …) so checker
    # routines downstream only see the simulated waveform prints.
    filtered_lines = [
        line for line in res.splitlines()
        if not line.startswith('INFO:')
    ]
    res = '\n'.join(filtered_lines)
    os.chdir(restore)
    return res

def parse_verilator_cycle(toks):
    '''Helper function to parse verilator dumped cycle'''
    # return int(toks[0]) // 100
    return int(toks[2][1:-4])

def parse_simulator_cycle(toks):
    '''Helper function to parse rust-simulator dumped cycle'''
    return int(toks[2][1:-4])

def has_verilator():
    '''Returns the path to Verilator or None if dependencies are missing'''
    # pylint: disable=global-statement
    global VERILATOR_CACHE
    if VERILATOR_CACHE is not None:
        return VERILATOR_CACHE

    verilator_root = os.environ.get('VERILATOR_ROOT')
    if not verilator_root:
        VERILATOR_CACHE = None
        return VERILATOR_CACHE
    if not os.path.isdir(verilator_root):
        VERILATOR_CACHE = None
        return VERILATOR_CACHE

    try:
        subprocess.run(
            [sys.executable, '-c', 'import pycde'],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (FileNotFoundError, subprocess.CalledProcessError):
        VERILATOR_CACHE = None
    else:
        VERILATOR_CACHE = 'verilator'
    return VERILATOR_CACHE

def create_dir(dir_path: str) -> None:
    """Create a directory if it doesn't exist.
    
    Args:
        dir_path: Path to the directory to create
        
    Raises:
        OSError: If directory creation fails due to permissions or disk space
    """
    os.makedirs(dir_path, exist_ok=True)

def namify(name: str) -> str:
    """Convert a name to a valid identifier.

    This matches the Rust function in src/backend/simulator/utils.rs
    """
    return ''.join(c if c.isalnum() or c == '_' else '_' for c in name)

def check_build_cache(src_dir: str, cache_key: str):
    """Check if cached build exists and is valid.

    Args:
        src_dir: Directory where cache file is stored
        cache_key: Combined IR hash + config hash key
    """

    cache_file = f'{src_dir}/.build_cache.json'
    if not os.path.exists(cache_file):
        return None
    try:
        with open(cache_file, 'r', encoding='utf-8') as f:
            cache = json.load(f)
        if cache.get('key') == cache_key:
            binary = cache.get('binary')
            if binary and os.path.exists(binary):
                return binary, cache.get('verilog')
    except (json.JSONDecodeError, KeyError, OSError):
        pass
    return None

def save_build_cache(src_dir: str, cache_key: str, binary: str, verilog: str):
    """Save build cache metadata.

    Args:
        src_dir: Directory where cache file will be stored
        cache_key: Combined IR hash + config hash key
        binary: Path to built simulator binary
        verilog: Path to generated verilog (optional)
    """
    cache_data = {
        'key': cache_key,
        'binary': str(binary),
        'verilog': str(verilog) if verilog else None
    }
    cache_file = f'{src_dir}/.build_cache.json'
    with open(cache_file, 'w', encoding='utf-8') as f:
        json.dump(cache_data, f)

__all__ = [
    # Type enforcement utilities
    'enforce_type', 'validate_arguments', 'check_type',
    # Existing utilities
    'identifierize', 'unwrap_operand', 'repo_path', 'package_path',
    'patch_fifo', 'run_simulator', 'build_simulator', 'get_simulator_binary_path',
    'run_verilator', 'parse_verilator_cycle',
    'parse_simulator_cycle', 'has_verilator', 'create_dir', 'namify',
    # Build caching
    'check_build_cache', 'save_build_cache'
]
