'''Downstream class is a special module that is combinational across multiple different
chronological modules.'''

from .base import ModuleBase, combinational_for
from ..block import Block
from ...builder import Singleton

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
        Singleton.repr_ident = 2
        var_id = self.as_operand()
        body = repr(self.body) if self.body is not None else ''
        ext = self._dump_externals()
        return f'''{ext}  #[{head}]
  {var_id} = module {self.name} {{
{body}
  }}
'''

    def __repr__(self):
        return self._repr_impl('downstream')

# Create the combinational decorator for Downstream
combinational = combinational_for(Downstream)
