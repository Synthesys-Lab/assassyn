'''The programming interfaces involing assassyn backends'''

import os
import tempfile
from pathlib import Path

from .builder import SysBuilder
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

    if real_config['verbose']:
        print(sys)

    proj_root = Path(real_config['path'])

    sys_dir = proj_root / sys.name

    make_existing_dir(sys_dir)

    simulator_manifest, verilog_path = codegen.codegen(sys, **real_config)

    return [simulator_manifest, verilog_path]
