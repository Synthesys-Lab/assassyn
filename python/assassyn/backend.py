"""The programming interfaces involing assassyn backends."""

from __future__ import annotations

import os
import inspect
import hashlib
import json
from pathlib import Path

from .builder import SysBuilder
from . import codegen
from . import utils

def config( # pylint: disable=too-many-arguments
        path='./workspace',
        resource_base=None,
        pretty_printer=True,
        verbose=True,
        simulator=True,
        verilog=False,
        sim_threshold=100,
        idle_threshold=100,
        fifo_depth=4,
        random=False,
        enable_cache=True):
    '''The helper function to dump the default configuration of elaboration.'''
    res = {
        'path': path,
        'resource_base': resource_base,
        'pretty_printer': pretty_printer,
        'verbose': verbose,
        'simulator': simulator,
        'verilog': verilog,
        'sim_threshold': sim_threshold,
        'idle_threshold': idle_threshold,
        'fifo_depth': fifo_depth,
        'random': random,
        'enable_cache': enable_cache
    }
    return res.copy()

def make_existing_dir(path):
    '''
    The helper function to create a directory if it does not exist.
    If it exists, it will print a warning message.
    '''
    try:
        os.makedirs(path)
    except FileExistsError:
        print(f'[WARN] {path} already exists, please make sure we did not override anything.')
    except Exception as e:
        raise e

def _generate_cache_key(sys_name: str, config_dict: dict) -> str:
    '''
    Generate a stable cache key from system name and configuration.

    Args:
        sys_name: Name of the system being built
        config_dict: Configuration dictionary

    Returns:
        A string that uniquely identifies this build configuration
    '''
    # Include only build-relevant parameters in cache key
    cache_params = {
        'system': sys_name,
        'simulator': config_dict.get('simulator', True),
        'verilog': config_dict.get('verilog', False),
        'sim_threshold': config_dict.get('sim_threshold'),
        'idle_threshold': config_dict.get('idle_threshold'),
        'fifo_depth': config_dict.get('fifo_depth'),
        'random': config_dict.get('random', False),
    }

    # Create a stable string representation and hash it
    cache_str = json.dumps(cache_params, sort_keys=True)
    cache_hash = hashlib.sha256(cache_str.encode()).hexdigest()[:12]

    return f"{sys_name}_{cache_hash}"

def elaborate(# pylint: disable=too-many-locals
        sys: SysBuilder, **kwargs):
    '''
    Invoke the elaboration process of the given system.
    Args:
        sys (SysBuilder): The assassyn system to be elaborated.
        path (Path): The directory where the Rust project will be dumped.
        pretty_printer (bool): Whether to run the Rust code formatter.
        verbose (bool): Whether dump the IR of the system to be elaborated.
        simulator (bool): Whether to generate the Rust code for the simulator.
        verilog (bool): Whether to generate the SystemVerilog code.
        idle_threshold (int): The threshold for the idle state to terminate the simulation.
        sim_threshold (int): The threshold for the simulation to terminate.
        **kwargs: The optional arguments that will be passed to the code generator.
    '''

    real_config = config()

    for k, v in kwargs.items():
        if k not in real_config:
            raise ValueError(f'Invalid config key: {k}')
        real_config[k] = v

    frame = inspect.stack()[1]
    caller_file = frame.filename
    source_dir = os.path.dirname(os.path.abspath(caller_file))

    ir_hash = hashlib.sha256(repr(sys).encode()).hexdigest()[:24]
    config_hash = _generate_cache_key(sys.name, real_config)
    cache_key = f"{ir_hash}_{config_hash}"

    # Check cache if source directory was detected and caching is enabled
    if source_dir and real_config.get('simulator', True) and real_config.get('enable_cache', True):
        cached = utils.check_build_cache(source_dir, cache_key)
        if cached:
            binary_path, verilog_path = cached
            print(f"[Cache Hit] Using cached build from {source_dir}")
            print(f"Binary: {binary_path}")
            if verilog_path:
                print(f"Verilog: {verilog_path}")
            return [binary_path, verilog_path]

    if real_config['verbose']:
        print(sys)

    proj_root = Path(real_config['path'])

    sys_dir = proj_root / sys.name

    make_existing_dir(sys_dir)

    # Update the path in config to point to the system directory
    real_config['path'] = str(sys_dir)

    # Generate code
    simulator_manifest, verilog_path = codegen.codegen(sys, **real_config)

    # Store cache info globally for build_simulator to use after building
    if source_dir and real_config.get('enable_cache', True):
        utils.CACHE_PENDING = (source_dir, cache_key, verilog_path)

    return [simulator_manifest, verilog_path]
