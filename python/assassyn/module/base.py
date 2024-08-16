'''The base class for the module definition.'''
from ..utils import identifierize

# pylint: disable=too-few-public-methods
class ModuleBase:
    '''The base class for the module definition.'''

    def as_operand(self):
        '''Dump the module as a right-hand side reference.'''
        return f'_{identifierize(self)}'

def name_ports_of_module(module, port_type):
    '''The helper function to name the ports of a module.'''
    for k, v in module.__dict__.items():
        if isinstance(v, port_type):
            v.name = k
            v.module = module
