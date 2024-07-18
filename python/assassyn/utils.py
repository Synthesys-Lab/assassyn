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
    print(path)
    cmd = ['make', 'main', '-j']
    subprocess.check_output(cmd).decode('utf-8')
    # TODO(@were): Fix this hardcoded Vtb later.
    cmd = ['./obj_dir/Vtb']
    res = subprocess.check_output(cmd).decode('utf-8')
    os.chdir(restore)
    return res

def parse_verilator_cycle(toks):
    return int(toks[0])

def parse_simulator_cycle(toks):
    return int(toks[1][2:-4])
