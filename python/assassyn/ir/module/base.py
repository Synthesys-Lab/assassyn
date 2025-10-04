'''The base class for the module definition.'''

from __future__ import annotations
import typing

from functools import wraps

from ...utils import namify, unwrap_operand, identifierize
from ...builder import ir_builder, Singleton
from ..expr import PureIntrinsic, Operand, Expr


# pylint: disable=too-few-public-methods, cyclic-import
class ModuleBase:
    '''The base class for the module definition.'''
    # Base class with no attributes of its own - attributes are added by derived classes

    _externals: typing.Dict[Expr, typing.List[Operand]] # External usage of this module

    def __init__(self):
        self._externals = {}

    def as_operand(self):
        '''Dump the module as a right-hand side reference.'''
        return f'_{namify(identifierize(self))}'

    @ir_builder
    def triggered(self):
        '''The frontend API for creating a triggered node,
        which checks if this module is triggered this cycle.
        NOTE: This operation is only usable in downstream modules.'''
        return PureIntrinsic(PureIntrinsic.MODULE_TRIGGERED, self)

    @property
    def externals(self):
        '''Expose the external interfaces of this module.'''
        return self._externals

    def add_external(self, operand: Operand):
        '''Add an external operand to this module.'''
        # pylint: disable=import-outside-toplevel
        from .module import Module
        from ..array import Array
        is_external = False
        if isinstance(operand, Operand):
            value = operand.value
            if isinstance(value, (Array, Module)):
                is_external = True
            if isinstance(value, Expr):
                is_external = value.parent.module != self
            if is_external:
                if value not in self._externals:
                    self._externals[value] = []
                self._externals[value].append(operand)

    def _dump_externals(self):
        # pylint: disable=import-outside-toplevel
        from ..block import Block
        res = ''
        for value, operands in self._externals.items():
            unwrapped = unwrap_operand(value)
            res = res + f'  // External: {unwrapped}\n'
            for operand in operands:
                if not isinstance(operand.user, Block):
                    res = res + f'  //  .usedby: {operand.user}\n'
                else:
                    res = res + f'  //  .usedby: condition::{operand.user.as_operand()}\n'
        return res

def combinational_for(module_type):
    '''A parameterizable decorator factory for marking a function as combinationa
      logic description.

    Args:
        module_type: The expected module type (Module or Downstream class).

    Returns:
        A decorator function that enforces the module type.
    '''
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            # pylint: disable=import-outside-toplevel
            from ..block import Block
            module_self = args[0]
            assert isinstance(module_self, module_type), \
                f"Expected {module_type.__name__}, got {type(module_self).__name__}"
            module_self.body = Block(Block.MODULE_ROOT)
            Singleton.builder.enter_context_of('module', module_self)
            with module_self.body:
                res = func(*args, **kwargs)
            Singleton.builder.exit_context_of('module')
            return res
        return wrapper
    return decorator
