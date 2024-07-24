'''The programming interfaces involing assassyn backends'''

import os
import subprocess
import tempfile

from .builder import SysBuilder
from . import utils
from . import codegen

def config( # pylint: disable=too-many-arguments
        path=tempfile.gettempdir(),
        resource_base=None,
        pretty_printer=True,
        verbose=True,
        finalized=False,
        simulator=True,
        verilog=False,
        sim_threshold=100,
        idle_threshold=100):
    '''The helper function to dump the default configuration of elaboration.'''
    res = {
        'path': path,
        'resource_base': resource_base,
        'pretty_printer': pretty_printer,
        'verbose': verbose,
        'finalized': finalized,
        'simulator': simulator,
        'verilog': verilog,
        'sim_threshold': sim_threshold,
        'idle_threshold': idle_threshold
    }
    return res.copy()

def dump_cargo_toml(path, name):
    '''
    Dump the Cargo.toml file for the Rust-implemented simulator

    Args:
        path (Path): The path to the directory where the Cargo.toml file will be dumped
        name (str): The name of the project
    '''
    toml = os.path.join(path, 'Cargo.toml')
    with open(toml, 'w', encoding='utf-8') as f:
        f.write('[package]\n')
        f.write(f'name = "{name}"\n')
        f.write('version = "0.0.0"\n')
        f.write('edition = "2021"\n')
        f.write('[dependencies]\n')
        f.write(f'eir = {{ path = \"{utils.repo_path()}/eir\" }}')
    return toml

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

def elaborate( # pylint: disable=too-many-arguments
               # pylint: disable=too-many-locals
        sys: SysBuilder,
        path=tempfile.gettempdir(),
        resource_base=None,
        pretty_printer=True,
        verbose=True,
        finalized=False,
        simulator=True,
        verilog=False,
        idle_threshold=100,
        sim_threshold=100):
    '''
    Invoke the elaboration process of the given system.

    Args:
        sys (SysBuilder): The assassyn system to be elaborated.
        path (Path): The directory where the Rust project will be dumped.
        pretty_printer (bool): Whether to run the Rust code formatter.
        verbose (bool): Whether dump the IR of the system to be elaborated.
        finalized (bool): Whether the system is finalized before feeding to this API.
        simulator (bool): Whether to generate the Rust code for the simulator.
        verilog (bool): Whether to generate the SystemVerilog code.
        idle_threshold (int): The threshold for the idle state to terminate the simulation.
        sim_threshold (int): The threshold for the simulation to terminate.
        **kwargs: The optional arguments that will be passed to the code generator.
    '''

    if not finalized:
        sys.finalize()

    if verbose:
        print(sys)

    sys_dir = os.path.join(path, sys.name)

    make_existing_dir(sys_dir)

    # Dump the Cargo.toml file
    toml = dump_cargo_toml(sys_dir, sys.name)
    # Dump the src directory
    make_existing_dir(os.path.join(sys_dir, 'src'))
    # Dump the assassyn IR builder
    with open(os.path.join(sys_dir, 'src/main.rs'), 'w', encoding='utf-8') as fd:
        raw = codegen.codegen(sys, simulator, verilog, idle_threshold, sim_threshold, resource_base)
        fd.write(raw)
    if pretty_printer:
        subprocess.run(['cargo', 'fmt', '--manifest-path', toml], cwd=sys_dir, check=True)
    subprocess.run(['cargo', 'run', '--release'], cwd=sys_dir, check=True)

    paths = []
    if simulator:
        paths.append(os.path.join(sys_dir, f'{sys.name}_simulator'))
    if verilog:
        paths.append(os.path.join(sys_dir, f'{sys.name}_verilog'))
    return paths[0] if len(paths) == 1 else paths
