'''The programming interfaces involing assassyn backends'''

import os
import subprocess
import tempfile

from .builder import SysBuilder
from . import utils
from . import codegen

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

def elaborate(
        sys: SysBuilder,
        path=tempfile.gettempdir(),
        pretty_printer=True,
        verbose=True,
        finalized=False,
        **kwargs):
    '''
    Invoke the elaboration process of the given system.

    Args:
        sys (SysBuilder): The assassyn system to be elaborated.
        path (Path): The directory where the Rust project will be dumped.
        pretty_printer (bool): Whether to run the Rust code formatter.
        verbose (bool): Whether dump the IR of the system to be elaborated.
        finalized (bool): Whether the system is finalized before feeding to this API.
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
        fd.write(codegen.codegen(sys, **kwargs))
    if pretty_printer:
        subprocess.run(['cargo', 'fmt', '--manifest-path', toml], cwd=sys_dir, check=True)
    subprocess.run(['cargo', 'run', '--release'], cwd=sys_dir, check=True)

    return os.path.join(sys_dir, f'simulator/{sys.name}')
