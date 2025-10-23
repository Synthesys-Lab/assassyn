'''The base class for the module definition.'''

from __future__ import annotations
import inspect
import typing

from functools import wraps

from ...utils import namify, unwrap_operand, identifierize
from ...builder import ir_builder, Singleton
from ...builder.rewrite_assign import rewrite_assign
from ..expr import Operand, Expr
from ..expr.intrinsic import PureIntrinsic


# pylint: disable=too-few-public-methods, cyclic-import
class ModuleBase:
    '''The base class for the module definition.'''
    # Base class with no attributes of its own - attributes are added by derived classes

    _externals: typing.Dict[Expr, typing.List[Operand]] # External usage of this module

    def __init__(self):
        self._externals = {}

    def as_operand(self):
        '''Dump the module as a right-hand side reference.'''
        name = getattr(self, 'name', None)
        if name:
            return name
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

    def add_external(self, operand: Operand) -> None:
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

def combinational_for(module_type):  # pylint: disable=too-many-statements
    '''Decorator factory for combinational module build functions with naming support.'''

    def decorator(func):  # pylint: disable=too-many-locals,too-many-statements
        # Use rewrite_assign to handle AST transformation
        new_func = rewrite_assign(func, adjust_lineno=True)

        @wraps(func)
        def wrapper(*args, **kwargs):
            # pylint: disable=import-outside-toplevel,cyclic-import
            from ..block import Block
            from ..array import Array

            module_self = args[0]
            assert isinstance(module_self, module_type), \
                f"Expected {module_type.__name__}, got {type(module_self).__name__}"

            module_self.body = Block(Block.MODULE_ROOT)
            module_self.body.parent = module_self
            module_self.body.module = module_self
            Singleton.builder.enter_context_of('module', module_self)
            Singleton.builder.enter_context_of('block', module_self.body)

            try:
                try:
                    bound = inspect.signature(new_func).bind(*args, **kwargs)
                    bound.apply_defaults()
                except TypeError:
                    bound = None

                if bound is not None:
                    for param_name, argument in bound.arguments.items():
                        if param_name == 'self':
                            continue
                        if isinstance(argument, Array):
                            # Only rename if doesn't have hierarchical name
                            # Arrays with underscores are module-scoped
                            current_name = getattr(argument, '_name', None)
                            # Preserve hierarchical names (with underscores,
                            # except 'array_xxxxx' pattern)
                            has_hierarchical_name = (
                                current_name and '_' in current_name
                                and not current_name.startswith('array_')
                            )
                            if not has_hierarchical_name:
                                argument.name = param_name

                return new_func(*args, **kwargs)
            finally:
                Singleton.builder.exit_context_of('block')
                Singleton.builder.exit_context_of('module')

        wrapper._is_combinational = True  # pylint: disable=protected-access
        wrapper._module_class = module_type  # pylint: disable=protected-access
        wrapper.__assassyn_original__ = new_func

        return wrapper

    return decorator
