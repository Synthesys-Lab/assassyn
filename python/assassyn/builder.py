'''The module provides the implementation of a class that is both IR builder and the system.'''

import inspect
import types
from decorator import decorator

class Singleton(type):
    '''The class maintains the global singleton instance of the system builder.'''
    builder = None
    repr_ident = None

@decorator
#pylint: disable=keyword-arg-before-vararg
def ir_builder(func, node_type=None, *args, **kwargs):
    '''The decorator annotates the function whose return value will be inserted into the AST.'''
    res = func(*args, **kwargs)
    module_symtab = Singleton.builder.is_direct_call(inspect.currentframe())
    Singleton.builder.insert_point[node_type].append(res)
    # This is to have a symbol table for the module currently being built,
    # so that we can name those named expressions.
    if module_symtab is not None:
        Singleton.builder.named_expr.append(res)
        Singleton.builder.module_symtab = module_symtab
    return res

#pylint: disable=too-many-instance-attributes
class SysBuilder:
    '''The class serves as both the system and the IR builder.'''

    def cleanup_symtab(self):
        '''Clean up the symbol table. Assign those named values to its identifier.'''
        value_dict = { id(v): v for v in self.named_expr }
        for k, v in self.module_symtab.items():
            if id(v) in value_dict:
                value_dict[id(v)].name = k

    def get_current_module(self):
        '''Get the current module being built.'''
        return self.cur_module

    def get_current_block(self):
        '''Get the current block being built.'''
        return self.cur_block

    def is_direct_call(self, frame: types.FrameType):
        '''If this function call is directly from the module.constructor'''
        upper_frame = frame.f_back.f_back
        if not upper_frame.f_locals.get('self') is self.cur_module:
            return None
        upper_frame = upper_frame.f_back
        caller = upper_frame.f_code.co_name
        if caller == 'combinational':
            return frame.f_back.f_back.f_locals
        return None

    def finalize(self):
        '''Finalize the modules underneath this system builder.'''
        for module in self.modules:
            module.finalized = True

    def __init__(self, name):
        self.name = name
        self.modules = []
        self.arrays = []
        self.insert_point = { 'array': self.arrays, 'expr': None, 'module': self.modules }
        self.cur_module = None
        self.cur_block = None
        self.builder_func = None
        self.module_symtab = None
        self.named_expr = []

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
        array = '  ' + '\n  '.join(repr(elem) for elem in self.arrays)
        return f'system {self.name} {{\n{array}\n\n{body}\n}}'
