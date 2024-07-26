'''The base class for the module definition.'''
from ..utils import identifierize

#pylint: disable=too-few-public-methods
class Timing:
    '''The enum class for the timing policy of a module.'''
    UNDEFINED = 0
    SYSTOLIC = 1
    BACKPRESSURE = 2

    def __init__(self, ty):
        self.ty = ty

    def __repr__(self):
        return ['undefined', 'systolic', 'backpressure'][self.ty]

class ModuleBase:
    '''The base class for the module definition.'''

    ATTR_EXPLICIT_FIFO = 0
    ATTR_DISABLE_ARBITER = 1
    ATTR_MEMORY = 2
    ATTR_TIMING = 3

    MODULE_ATTR_STR = {
      ATTR_EXPLICIT_FIFO: 'explicit_fifo',
      ATTR_DISABLE_ARBITER: 'no_arbiter',
      ATTR_MEMORY: 'memory',
      ATTR_TIMING: 'timing',
    }

    def __init__(self, is_explicit_fifo, timing, disable_arbiter_rewrite):
        self._attrs = {}
        self.parse_attrs(is_explicit_fifo, timing, disable_arbiter_rewrite)

    def parse_attrs(self, is_explicit_fifo, timing, disable_arbiter_rewrite):
        '''The helper function to parse the attributes.'''
        self._attrs[ModuleBase.ATTR_EXPLICIT_FIFO] = is_explicit_fifo
        self._attrs[ModuleBase.ATTR_TIMING] = Timing(timing)
        self._attrs[ModuleBase.ATTR_DISABLE_ARBITER] = disable_arbiter_rewrite

    @property
    def is_systolic(self):
        '''The helper function to get if this module is systolic.'''
        value = self._attrs.get(ModuleBase.ATTR_TIMING, Timing(Timing.UNDEFINED)).ty
        return value == Timing.SYSTOLIC

    @property
    def disable_arbiter_rewrite(self):
        '''The helper function to get the no-arbiter setting.'''
        return self._attrs.get(ModuleBase.ATTR_DISABLE_ARBITER, False)

    @property
    def is_explicit_fifo(self):
        '''The helper function to get the implicit FIFO setting.'''
        return self._attrs.get(ModuleBase.ATTR_EXPLICIT_FIFO, False)

    def as_operand(self):
        '''Dump the module as a right-hand side reference.'''
        return f'_{identifierize(self)}'
