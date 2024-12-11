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
        simulator=True,
        verilog=False,
        sim_threshold=100,
        idle_threshold=100,
        fifo_depth=4,
        random=False):
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
        'random': random
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
        f.write('prost = "0.13"\n')
        f.write('bytes = "1.5"\n')
        f.write(f'assassyn = {{ path = \"{utils.repo_path()}\" }}\n')
        f.write('[build-dependencies]\n')
        f.write('prost-build = "0.13"\n')
    return toml

def dump_build_rs(path):
    '''
    Dump the build.rs file for proto compilation

    Args:
        path (Path): The path to the directory where the build.rs file will be dumped
    '''
    build_rs = os.path.join(path, 'build.rs')
    with open(build_rs, 'w', encoding='utf-8') as f:
        f.write('fn main() {\n')
        f.write('    prost_build::compile_protos(&["src/create.proto"],\n')
        f.write('                               &["src"])\n')
        f.write('        .unwrap();\n')
        f.write('}\n')

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
        simulator=True,
        verilog=False,
        idle_threshold=100,
        sim_threshold=100,
        fifo_depth=4,
        random=False):
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

    if verbose:
        print(sys)

    sys_dir = os.path.join(path, sys.name)

    make_existing_dir(sys_dir)

    # Dump the Cargo.toml file
    toml = dump_cargo_toml(sys_dir, sys.name)
    
    # Dump build.rs file
    dump_build_rs(sys_dir)
    
    # Create src directory
    src_dir = os.path.join(sys_dir, 'src')
    make_existing_dir(src_dir)
    
    # Copy proto file
    proto_src = os.path.join(utils.repo_path(), 'python/assassyn/create.proto')
    proto_dst = os.path.join(src_dir, 'create.proto')
    try:
        import shutil
        shutil.copy2(proto_src, proto_dst)
    except Exception as e:
        print(f'[ERROR] Failed to copy proto file: {e}')
        raise e
    
    # Generate and write code
    random_sims = "false"
    if random:
        random_sims = "true"
    rust_code, serialized_ops = codegen.codegen(
        sys, simulator, verilog,
        idle_threshold, sim_threshold, random_sims,
        resource_base, fifo_depth
    )
    
    # Dump the assassyn IR builder
    with open(os.path.join(sys_dir, 'src/main.rs'), 'w', encoding='utf-8') as fd:
        fd.write(rust_code)
    
    # Dump the operation serial
    with open(os.path.join(sys_dir, 'src/create.pb'), 'wb') as fd:
        fd.write(serialized_ops)
        
    if pretty_printer:
        subprocess.run(['cargo', 'fmt', '--manifest-path', toml], cwd=sys_dir, check=True)
    subprocess.run(['cargo', 'run', '--release'], cwd=sys_dir, check=True)

    simulator_path = None
    verilog_path = None
    if simulator:
        simulator_path = os.path.join(sys_dir, f'{sys.name}_simulator')
    if verilog:
        verilog_path = os.path.join(sys_dir, f'{sys.name}_verilog')

    return [simulator_path, verilog_path]
