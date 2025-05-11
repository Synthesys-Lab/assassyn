'''The base class for the module definition.'''
from ..utils import identifierize
from ..builder import ir_builder
from ..expr import PureInstrinsic


# pylint: disable=too-few-public-methods
class ModuleBase:
    '''The base class for the module definition.'''
    # Base class with no attributes of its own - attributes are added by derived classes

    def __init__(self):
        pass

    def as_operand(self):
        '''Dump the module as a right-hand side reference.'''
        return f'_{identifierize(self)}'

    @ir_builder
    def triggered(self):
        '''The frontend API for creating a triggered node,
        which checks if this module is triggered this cycle.
        NOTE: This operation is only usable in downstream modules.'''
        return PureInstrinsic(PureInstrinsic.MODULE_TRIGGERED, self)
