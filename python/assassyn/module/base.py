'''The base class for the module definition.'''
from ..utils import identifierize
from ..builder import ir_builder
from ..expr import PureInstrinsic


# pylint: disable=too-few-public-methods
class ModuleBase:
    '''The base class for the module definition.'''

    def as_operand(self):
        '''Dump the module as a right-hand side reference.'''
        return f'_{identifierize(self)}'

    @ir_builder(node_type='expr')
    def triggered(self):
        '''The frontend API for creating a triggered node,
        which checks if this module is triggered this cycle.
        NOTE: This operation is only usable in downstream modules.'''
        return PureInstrinsic(PureInstrinsic.MODULE_TRIGGERED, self)

def name_ports_of_module(module, port_type):
    '''The helper function to name the ports of a module.'''
    for k, v in module.__dict__.items():
        if isinstance(v, port_type):
            v.name = k
            v.module = module
