'''The base class for the module definition.'''

from __future__ import annotations
import ast
import inspect
import textwrap
import typing

from functools import wraps

from ...utils import namify, unwrap_operand, identifierize
from ...builder import ir_builder, Singleton
from ...builder.rewrite_assign import rewrite_assign, __assassyn_assignment__ as _assignment_fn
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
        semantic = getattr(self, "__assassyn_semantic_name__", None)
        if semantic:
            return semantic
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
    '''Decorator factory for combinational module build functions with naming support.'''

    def decorator(func):
        try:
            source = textwrap.dedent(inspect.getsource(func))
            tree = ast.parse(source)
            func_def = tree.body[0]

            rewritten_func_def = rewrite_assign(func_def)
            rewritten_func_def.decorator_list = []

            tree.body[0] = rewritten_func_def
            ast.fix_missing_locations(tree)

            namespace = func.__globals__
            had_assignment_hook = '__assassyn_assignment__' in namespace
            previous_hook = namespace.get('__assassyn_assignment__')
            namespace['__assassyn_assignment__'] = _assignment_fn

            code = compile(tree, func.__code__.co_filename, 'exec')
            exec(code, namespace)  # pylint: disable=exec-used
            new_func = namespace[func.__name__]

            if had_assignment_hook:
                namespace['__assassyn_assignment__'] = previous_hook
        except Exception as exc:  # pylint: disable=broad-except
            # Fallback to original function if rewriting fails
            # Deferred import to avoid cycles at module import time.
            import sys  # pylint: disable=import-outside-toplevel
            print(f"Warning: AST rewriting failed for {func.__name__}: {exc}", file=sys.stderr)
            new_func = func

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
