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
