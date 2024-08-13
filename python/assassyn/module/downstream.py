'''Downstream class is a special module that is combinational across multiple different
chronological modules.'''

from decorator import decorator

from .base import ModuleBase, name_ports_of_module
from ..block import Block
from ..value import Optional
from ..builder import Singleton

@decorator
def constructor(func, *args, **kwargs):
    '''Constructor decorator for the Downstream class.'''
    func(*args, **kwargs)
    builder = Singleton.builder
    module_self = args[0]
    builder.downstreams.append(module_self)
    name_ports_of_module(module_self, Optional)


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
        self.name = type(self).__name__
        self.name = self.name + '_' + self.as_operand()
        self.body = None

    @property
    def ports(self):
        '''The helper function to get all the ports in the module.'''
        return [v for _, v in self.__dict__.items() if isinstance(v, Optional)]

    def __repr__(self):
        var_id = self.as_operand()
        ports = ',\n    '.join(f'{p.name}: {p}' for p in self.ports)
        body = self.body.__repr__() if self.body is not None else ''
        return f'''  #[downstream]
  {var_id} = module {self.name} {{
    {ports}
  }} {{
{body}
  }}
'''
