'''The untilities for the project'''

import os
import subprocess

def repo_path():
    '''Get the path to assassyn repository'''
    return os.environ['ASSASSYN_HOME']

def run_simulator(path):
    '''The helper function to run the simulator'''
    cmd = ['cargo', 'run', '--manifest-path', path + '/Cargo.toml', '--release']
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

def verilator_path():
    '''Returns the path to Verilator or None if VERILATOR_ROOT is not set'''
    verilator_root = os.environ.get('VERILATOR_ROOT')
    if verilator_root and os.path.isdir(verilator_root):
        return 'verilator'
    return None
