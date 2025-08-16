'''The untilities for the project'''

# import timeit
import os
import subprocess


def identifierize(obj):
    '''The helper function to get the identifier of the given object. You can change `id_slice`
    to tune the length of the identifier. The default is slice(-5:-1).'''
    # pylint: disable=import-outside-toplevel
    from .builder import Singleton
    return hex(id(obj))[Singleton.id_slice]

def unwrap_operand(node):
    """Unwrap the operand from the node.

    This is a helper function to get the operand from the node.
    """
    # pylint: disable=import-outside-toplevel
    from .ir.expr import Operand
    if isinstance(node, Operand):
        return node.value
    return node

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

def _cmd_wrapper(cmd):
    return subprocess.check_output(cmd).decode('utf-8')

def run_simulator(manifest_path, offline=False, release=True):
    '''The helper function to run the simulator'''
    cmd = ['cargo', 'run', '--manifest-path', manifest_path]
    if offline:
        cmd += ['--offline']
    if release:
        cmd += ['--release']
    res = _cmd_wrapper(cmd)
    return res

def run_verilator(path):
    '''The helper function to run the verilator'''
    # restore = os.getcwd()
    os.chdir(path)
    cmd_design = ['python3', 'design.py']
    subprocess.check_output(cmd_design)

    cmd_tb = ['python3', 'tb.py']
    res = _cmd_wrapper(cmd_tb)
    # cmd = ['make', 'main', '-j']
    # subprocess.check_output(cmd).decode('utf-8')
    # # TODO(@were): Fix this hardcoded Vtb later.
    # cmd = ['./obj_dir/Vtb']
    # res = _cmd_wrapper(cmd)
    # if count_time:
    #     a = timeit.timeit(lambda: _cmd_wrapper(cmd), number=5)
    #     os.chdir(restore)
    #     return (res, a)
    # os.chdir(restore)
    return res

def parse_verilator_cycle(toks):
    '''Helper function to parse verilator dumped cycle'''
    # return int(toks[0]) // 100
    return int(toks[2][1:-4])

def parse_simulator_cycle(toks):
    '''Helper function to parse rust-simulator dumped cycle'''
    return int(toks[2][1:-4])

def has_verilator():
    '''Returns the path to Verilator or None if VERILATOR_ROOT is not set'''
    verilator_root = os.environ.get('VERILATOR_ROOT')
    if verilator_root and os.path.isdir(verilator_root):
        return 'verilator'
    return None

def create_and_clean_dir(dir_path: str):
    """Create a directory and clear its contents if it already exists."""
    # Create the directory
    os.makedirs(dir_path, exist_ok=True)

def namify(name: str) -> str:
    """Convert a name to a valid identifier.

    This matches the Rust function in src/backend/simulator/utils.rs
    """
    return ''.join(c if c.isalnum() or c == '_' else '_' for c in name)
