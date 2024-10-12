'''The untilities for the project'''

import os
import subprocess

def identifierize(obj):
    '''The helper function to get the identifier of the given object. You can change `id_slice`
    to tune the length of the identifier. The default is slice(-5:-1).'''
    # pylint: disable=import-outside-toplevel
    from .builder import Singleton
    return hex(id(obj))[Singleton.id_slice]

PATH_CACHE = None

def repo_path():
    '''Get the path to assassyn repository'''
    # pylint: disable=global-statement
    global PATH_CACHE
    if PATH_CACHE is None:
        PATH_CACHE = os.environ['ASSASSYN_HOME']
    return PATH_CACHE

def package_path():
    '''Get the path to this python package'''
    return repo_path() + '/python/assassyn'

def run_simulator(path):
    '''The helper function to run the simulator'''
    cmd = ['cargo', 'run', '--manifest-path', path + '/Cargo.toml', '--release', '--offline']
    return subprocess.check_output(cmd).decode('utf-8')

def run_verilator(path):
    '''The helper function to run the verilator'''
    restore = os.getcwd()
    os.chdir(path)
    cmd = ['make', 'main', '-j']
    subprocess.check_output(cmd).decode('utf-8')
    # TODO(@were): Fix this hardcoded Vtb later.
    cmd = ['./obj_dir/Vtb']
    res = subprocess.check_output(cmd).decode('utf-8')
    os.chdir(restore)
    return res

def parse_verilator_cycle(toks):
    '''Helper function to parse verilator dumped cycle'''
    return int(toks[0]) // 100

def parse_simulator_cycle(toks):
    '''Helper function to parse rust-simulator dumped cycle'''
    return int(toks[2][1:-4])

def has_verilator():
    '''Returns the path to Verilator or None if VERILATOR_ROOT is not set'''
    verilator_root = os.environ.get('VERILATOR_ROOT')
    if verilator_root and os.path.isdir(verilator_root):
        return 'verilator'
    return None
