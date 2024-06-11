import os
import shutil
import subprocess
import tempfile

from .builder import SysBuilder
from . import utils
from . import codegen

def dump_cargo_toml(path, name):
    toml = os.path.join(path, 'Cargo.toml')
    with open(toml, 'w') as f:
        f.write('[package]\n')
        f.write('name = "%s"\n' % name)
        f.write('version = "0.0.0"\n')
        f.write('edition = "2021"\n')
        f.write('[dependencies]\n')
        f.write('eir = { path = \"%s/eir\" }' % utils.repo_path())

def make_existing_dir(path):
    try:
        os.makedirs(path)
    except FileExistsError:
        print('[WARN] %s already exists, please make sure we did not override anything.' % path)
    except Exception as e:
        raise e

def elaborate(sys: SysBuilder, path=tempfile.gettempdir(), **kwargs):

    sys_dir = os.path.join(path, sys.name)

    make_existing_dir(sys_dir)

    # Dump the Cargo.toml file
    dump_cargo_toml(sys_dir, sys.name)
    # Dump the src directory
    make_existing_dir(os.path.join(sys_dir, 'src'))
    # Dump the assassyn IR builder
    with open(os.path.join(sys_dir, 'src/main.rs'), 'w') as fd:
        fd.write(codegen.codegen(sys))
    subprocess.run(['cargo', 'run', '--release'], cwd=sys_dir)

    return os.path.join(sys_dir, 'simulator/%s' % sys.name)
