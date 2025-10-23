'''Downstream class is a special module that is combinational across multiple different
chronological modules.'''

from .base import ModuleBase, combinational_for
from ..block import Block
from ...builder import Singleton

class Downstream(ModuleBase):
    '''Downstream class implementation.'''

    _name: str  # Internal name storage
    body: Block  # Body of the downstream module

    @property
    def name(self) -> str:
        '''Get the name of the downstream module.'''
        return self._name

    @name.setter
    def name(self, value: str):
        '''Set the name for IR generation.'''
        self._name = value

    def __init__(self):
        super().__init__()
        base_name = type(self).__name__

        # Use naming manager if available for consistent naming
        manager = getattr(Singleton, 'naming_manager', None)
        if manager is not None:
            assigned_name = manager.assign_name(self, base_name)
            self._name = assigned_name
        else:
            self._name = base_name + self.as_operand()

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
