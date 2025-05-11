'''The module provides the implementation of a class that is both IR builder and the system.'''
import os
import site
import inspect
from decorator import decorator

@decorator
def ir_builder(func, *args, **kwargs):
    '''The decorator annotates the function whose return value will be inserted into the AST.'''
    res = func(*args, **kwargs)
    #pylint: disable=cyclic-import,import-outside-toplevel
    from .const import Const
    from .utils import package_path

    if not isinstance(res, Const):
        Singleton.builder.insert_point.append(res)

    package_dir = os.path.abspath(package_path())

    Singleton.initialize_dirs_to_exclude()
    for i in inspect.stack()[2:]:
        fname, lineno = i.filename, i.lineno
        fname_abs = os.path.abspath(fname)

        if not fname_abs.startswith(package_dir) \
            and not any(fname_abs.startswith(exclude_dir) \
                         for exclude_dir in Singleton.all_dirs_to_exclude):
            res.loc = f'{fname}:{lineno}'
            break
    assert hasattr(res, 'loc')
    return res


#pylint: disable=too-many-instance-attributes
class SysBuilder:
    '''The class serves as both the system and the IR builder.'''

    name: str  # Name of the system
    modules: list  # List of modules
    downstreams: list  # List of downstream modules
    arrays: list  # List of arrays
    _ctx_stack: dict  # Stack for context tracking
    _exposes: dict  # Dictionary of exposed nodes

    @property
    def current_module(self):
        '''Get the current module being built.'''
        return None if not self._ctx_stack['module'] else self._ctx_stack['module'][-1]

    @property
    def current_block(self):
        '''Get the current block being built.'''
        return None if not self._ctx_stack['block'] else self._ctx_stack['block'][-1]

    @property
    def insert_point(self):
        '''Get the insert point.'''
        return self.current_block.body

    def enter_context_of(self, ty, entry):
        '''Enter the context of the given type.'''
        self._ctx_stack[ty].append(entry)

    def exit_context_of(self, ty):
        '''Exit the context of the given type.'''
        self._ctx_stack[ty].pop()

    def __init__(self, name):
        self.name = name
        self.modules = []
        self.downstreams = []
        self.arrays = []
        self._ctx_stack = {'module': [], 'block': []}
        self._exposes = {}

    def expose_on_top(self, node, kind=None):
        '''Expose the given node in the top function with the given kind.'''
        self._exposes[node] = kind

    @property
    def exposed_nodes(self):
        '''Get the exposed nodes.'''
        return self._exposes

    def __enter__(self):
        '''Designate the scope of this system builder.'''
        assert Singleton.builder is None
        Singleton.builder = self
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        '''Leave the scope of this system builder.'''
        assert Singleton.builder is self
        Singleton.builder = None

    def __repr__(self):
        body = '\n\n'.join(map(repr, self.modules))
        body = body + '\n\n' + '\n\n'.join(map(repr, self.downstreams))
        array = '  ' + '\n  '.join(repr(elem) for elem in self.arrays)
        return f'system {self.name} {{\n{array}\n\n{body}\n}}'

class Singleton(type):
    '''The class maintains the global singleton instance of the system builder.'''
    builder: SysBuilder = None  # Global singleton instance of the system builder
    repr_ident: int = None  # Indentation level for string representation
    id_slice: slice = slice(-6, -1)  # Slice for identifiers
    with_py_loc: bool = False  # Whether to include Python location in string representation
    all_dirs_to_exclude: list = []  # Directories to exclude for stack inspection

    @classmethod
    def initialize_dirs_to_exclude(mcs):
        '''Initialize the directories to exclude if not already initialized.'''
        if not mcs.all_dirs_to_exclude:
            site_package_dirs = site.getsitepackages()
            user_site_package_dir = site.getusersitepackages()
            mcs.all_dirs_to_exclude = site_package_dirs + [user_site_package_dir]
