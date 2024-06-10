import os
import shutil
import subprocess
import tempfile

from .builder import SysBuilder
from . import utils

def dump_cargo_toml(path, name):
    toml = os.path.join(path, 'Cargo.toml')
    with open(toml, 'w') as f:
        f.write('[package]\n')
        f.write('name = "%s"\n' % name)
        f.write('version = "0.0.0"\n')
        f.write('edition = "2024"\n\n')
        f.write('[dependencies]\n')
        f.write('eir = { path = %s/eir }' % utils.repo_path())


def build(sys: SysBuilder, path=tempfile.gettempdir(), **kwargs):

    sys_dir = os.path.join(path, sys.name)

    try:
        os.makedirs(sys_dir)
    except FileExistsError:
        pass
    except Exception as e:
        raise e

    for f in os.listdir(sys_dir):
        to_remove = os.path.join(sys_dir, f)
        (shutil.rmtree if os.path.isdir(to_remove) else os.remove)(to_remove)

    # Initialize the cargo project, including toml and folders.
    subprocess.run(['cargo', 'init', sys_dir])
    # "Correct" the Cargo.toml file
    dump_cargo_toml(sys_dir, sys.name)
    # Dump the assassyn IR builder
    dump_sys(sys, sys_dir)
