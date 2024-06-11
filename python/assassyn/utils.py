import os
import subprocess

def repo_path():
    return os.environ['ASSASSYN_HOME']

def run_simulator(path):
    cmd = ['cargo', 'run', '--manifest-path', path + '/Cargo.toml', '--release']
    return subprocess.check_output(cmd).decode('utf-8')

