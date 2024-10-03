'''The module provides the implementation of a class that is both IR builder and the system.'''

import inspect
from decorator import decorator

class Singleton(type):
    '''The class maintains the global singleton instance of the system builder.'''
    builder = None
    repr_ident = None
    id_slice = slice(-6, -1)
    with_py_loc = False

@decorator
def ir_builder(func, *args, **kwargs):
    '''The decorator annotates the function whose return value will be inserted into the AST.'''
    res = func(*args, **kwargs)
    #pylint: disable=cyclic-import,import-outside-toplevel
    from .const import Const
    if not isinstance(res, Const):
        Singleton.builder.insert_point.append(res)
    stack_entry = inspect.stack()[2]
    res.loc = (stack_entry.filename, stack_entry.lineno, stack_entry.function)
    return res

#pylint: disable=too-many-instance-attributes
class SysBuilder:
    '''The class serves as both the system and the IR builder.'''

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
