'''Downstream class is a special module that is combinational across multiple different
chronological modules.'''

from decorator import decorator

from .base import ModuleBase
from ..block import Block
from ..builder import Singleton

@decorator
def combinational(
        func,
        *args,
        **kwargs):
    '''A decorator for marking a function as combinational logic description.'''
    module_self = args[0]
    assert isinstance(module_self, Downstream)
    Singleton.builder.enter_context_of('module', module_self)
    module_self.body = Block(Block.MODULE_ROOT)
    with module_self.body:
        res = func(*args, **kwargs)
    Singleton.builder.exit_context_of('module')
    return res

class Downstream(ModuleBase):
    '''Downstream class implementation.'''

    name: str  # Name of the downstream module
    body: Block  # Body of the downstream module

    def __init__(self):
        super().__init__()
        self.name = type(self).__name__
        self.name = self.name + self.as_operand()
        self.body = None

        Singleton.builder.downstreams.append(self)

    def _repr_impl(self, head):
        var_id = self.as_operand()
        body = repr(self.body) if self.body is not None else ''
        return f'''  #[{head}]
  {var_id} = module {self.name} {{
{body}
  }}
'''

    def __repr__(self):
        return self._repr_impl('downstream')
