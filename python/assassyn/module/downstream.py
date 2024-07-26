'''Downstream class is a special module that is combinational across multiple different
chronological modules.'''

from decorator import decorator

from .base import ModuleBase, name_ports_of_module
from ..value import Optional

def constructor(func, *args, **kwargs):
    '''Constructor decorator for the Downstream class.'''
    builder = Singleton.builder
    module_self = args[0]
    builder.insert_point['module'].append(module_self)
    func(*args, **kwargs)
    name_ports_of_module(module_self, Optional)

def combinatorial(func, *args, **kwargs):
    '''Combinatorial decorator for the Downstream class.'''
    return func(*args, **kwargs)

@decorator
#pylint: disable=keyword-arg-before-vararg
def combinational(
        func,
        *args,
        **kwargs):
    '''A decorator for marking a function as combinational logic description.'''
    module_self = args[0]
    assert isinstance(module_self, Downstream)
    Singleton.builder.cur_module = module_self
    Singleton.builder.builder_func = func
    module_self.body = Block(Block.MODULE_ROOT)
    with module_self.body:
        res = func(*args, **kwargs)
    Singleton.builder.cleanup_symtab()
    Singleton.builder.cur_module = None
    return res

class Downstream(ModuleBase):
    '''Downstream class implementation.'''

    def __init__(self):
        super().__init__()
