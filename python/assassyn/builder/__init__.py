'''The module provides the implementation of a class that is both IR builder and the system.'''

#pylint: disable=cyclic-import,duplicate-code

from __future__ import annotations

import functools
import inspect
import os
import site
import typing
from .naming_manager import (
    NamingManager,
)
from .rewrite_assign import rewrite_assign
from .type_oriented_namer import TypeOrientedNamer
from .unique_name import UniqueNameCache

if typing.TYPE_CHECKING:
    from ..ir.array import Array, ArrayRead
    from ..ir.dtype import DType
    from ..ir.module import Module
    from ..ir.value import Value

__all__ = [
    # Core components
    'UniqueNameCache',
    'TypeOrientedNamer',
    'NamingManager',

    # Decorators
    'rewrite_assign',

]


def ir_builder(func=None):
    '''Decorator that records builder metadata and injects IR nodes into the AST.'''

    def _decorate(target):
        @functools.wraps(target)
        def _wrapper(*args, **kwargs):  # pylint: disable=too-many-nested-blocks,too-many-locals
            res = target(*args, **kwargs)

            # This indicates this res is handled somewhere else, so we do not need to rehandle it
            if res is None:
                return res

            #pylint: disable=cyclic-import,import-outside-toplevel
            from ..ir.const import Const
            from ..utils import package_path
            from ..ir.expr import Expr

            builder = Singleton.peek_builder()
            manager = builder.naming_manager
            is_expr = isinstance(res, Expr)
            already_materialized = is_expr and getattr(res, 'parent', None) is not None

            if is_expr and not already_materialized:
                manager.push_value(res)

            if not isinstance(res, Const):
                if is_expr and not already_materialized:
                    current_module = builder.current_module
                    res.parent = current_module
                    for operand in res.operands:
                        current_module.add_external(operand)
                if not already_materialized:
                    builder.insert_point.append(res)

            package_dir = os.path.abspath(package_path())

            Singleton.initialize_dirs_to_exclude()
            for i in inspect.stack()[1:]:  # pylint: disable=too-many-nested-blocks
                fname, lineno = i.filename, i.lineno
                fname_abs = os.path.abspath(fname)

                if not fname_abs.startswith(package_dir) \
                    and not any(
                        fname_abs.startswith(exclude_dir)
                        for exclude_dir in Singleton.all_dirs_to_exclude
                    ):
                    res.loc = f'{fname}:{lineno}'

                    break
            assert hasattr(res, 'loc')
            return res

        return _wrapper

    if func is None:
        return _decorate
    return _decorate(func)


#pylint: disable=too-many-instance-attributes
class PredicateFrame:  # pylint: disable=too-few-public-methods
    '''Per-predicate frame containing the condition and its array-read cache.'''
    cond: Value
    carry: Value
    array_cache: dict[tuple[Array, Value], ArrayRead]

    def __init__(self, cond: Value, carry: Value):
        self.cond = cond
        self.carry = carry
        self.array_cache = {}

    def get_cached_read(self, array: Array, index: Value) -> ArrayRead | None:
        '''Probe this frame's cache for an existing read operation.

        @param array The array being read from.
        @param index The index being read at.
        @return The cached ArrayRead if found, None otherwise.
        '''
        return self.array_cache.get((array, index))

    def cache_read(self, array: Array, index: Value, read: ArrayRead) -> None:
        '''Store an array read operation in this frame's cache.

        @param array The array being read from.
        @param index The index being read at.
        @param read The ArrayRead to cache.
        '''
        self.array_cache[(array, index)] = read

    def has_cached_read(self, array: Array, index: Value) -> bool:
        '''Check if a read operation is cached in this frame.

        @param array The array being read from.
        @param index The index being read at.
        @return True if cached, False otherwise.
        '''
        return (array, index) in self.array_cache


class ModuleContext:  # pylint: disable=too-few-public-methods
    '''Module-scoped context record holding module and its predicate stack.'''

    module: Module
    cond_stack: list[PredicateFrame]

    def __init__(self, module: Module):
        self.module = module
        self.cond_stack = []


class SysBuilder:
    '''The class serves as both the system and the IR builder.'''

    name: str  # Name of the system
    modules: typing.List[Module]  # List of modules
    downstreams: list  # List of downstream modules
    arrays: typing.List[Array]  # List of arrays
    _module_stack: list[ModuleContext]  # Stack for module context tracking
    _exposes: dict  # Dictionary of exposed nodes
    line_expression_tracker: dict  # Dictionary of line expression tracker
    naming_manager: NamingManager  # Naming manager

    @property
    def current_module(self):
        '''Get the current module being built.'''
        module_stack = self._module_stack
        if not module_stack:
            raise RuntimeError('Module context stack is empty')
        return module_stack[-1].module

    @property
    def current_body(self):
        '''Get the current module body being built.'''
        module = self.current_module
        body = getattr(module, 'body', None)
        if body is None:
            raise RuntimeError(f'Module {module!r} has no active body')
        return body

    @property
    def insert_point(self):
        '''Get the insert point.'''
        return self.current_body

    # Predicate stack helpers (per current module context)
    def get_predicate_stack(self):
        '''Get the current module's predicate stack.'''
        module_stack = self._module_stack
        return [] if not module_stack else module_stack[-1].cond_stack

    def current_predicate_carry(self):
        '''Return the cumulative predicate for the current context.'''
        stack = self.get_predicate_stack()
        if not stack:
            # pylint: disable=import-outside-toplevel
            from ..ir.dtype import Bits
            return Bits(1)(1)
        return stack[-1].carry

    def reuse_array_read(self, array, index, factory):
        '''Reuse a cached array read or materialize a new one via ``factory``.'''
        stack = self.get_predicate_stack()

        for frame in reversed(stack):
            cached = frame.get_cached_read(array, index)
            if cached is not None:
                return cached

        read = factory()

        if stack:
            stack[-1].cache_read(array, index, read)

        return read

    def push_predicate(self, cond):
        '''Push a predicate into current module's predicate stack.'''
        stack = self.get_predicate_stack()
        if not stack:
            carry = cond
        else:
            from ..ir.expr import comm  # pylint: disable=import-outside-toplevel
            carry = comm.and_(stack[-1].carry, cond)
        frame = PredicateFrame(cond, carry)
        stack.append(frame)

    def pop_predicate(self):
        '''Pop a predicate from current module's predicate stack.'''
        stack = self.get_predicate_stack()
        assert stack, 'Predicate stack underflow'
        stack.pop()

    def enter_context_of(self, module: Module) -> None:
        '''Enter the context of the given module.'''
        if module is None:
            raise RuntimeError('Cannot enter context of None')
        body = getattr(module, 'body', None)
        if body is None:
            raise RuntimeError(f'Module {module!r} has no body before entering context')
        self._module_stack.append(ModuleContext(module))

    def exit_context_of(self) -> ModuleContext:
        '''Exit the current module context.'''
        if not self._module_stack:
            raise RuntimeError('Module context stack is empty')
        ctx = self._module_stack.pop()
        if ctx.cond_stack:
            msg = 'Predicate stack not empty on module exit: ' + repr(ctx.cond_stack[-1].cond)
            assert False, msg
        return ctx

    def has_driver(self):
        '''Check if the system has a driver module.'''
        for i in self.modules:
            if i.__class__.__name__ == 'Driver':
                return True
        return False

    def has_module(self, name):
        '''Check if a module with the given name exists.'''
        for i in self.modules:
            if i.name == name:
                return i
        return None

    def __init__(self, name):
        self.name = name
        self.modules = []
        self.downstreams = []
        self.arrays = []
        self._module_stack = []
        self._exposes = {}
        self.line_expression_tracker = {}
        self.naming_manager = NamingManager()
        self._reset_caches()

    def expose_on_top(self, node, kind=None):
        '''Expose the given node in the top function with the given kind.'''
        self._exposes[node] = kind

    @property
    def exposed_nodes(self):
        '''Get the exposed nodes.'''
        return self._exposes

    def _reset_caches(self):
        '''Initialise or clear per-builder caches.'''
        self.const_cache = {}

    def __enter__(self):
        '''Designate the scope of this system builder.'''
        Singleton.set_builder(self)
        Singleton.line_expression_tracker = self.line_expression_tracker
        Singleton.naming_manager = self.naming_manager
        self._reset_caches()
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        '''Leave the scope of this system builder.'''
        assert Singleton.peek_builder() is self
        Singleton.set_builder(None)
        Singleton.line_expression_tracker = None
        Singleton.naming_manager = None
        self._reset_caches()

    def __repr__(self):
        body = '\n\n'.join(map(repr, self.modules))
        body = body + '\n\n' + '\n\n'.join(map(repr, self.downstreams))
        array = '  ' + '\n  '.join(repr(elem) for elem in self.arrays)
        return f'system {self.name} {{\n{array}\n\n{body}\n}}'

class Singleton(type):
    '''The class maintains the global singleton instance of the system builder.'''
    _builder: SysBuilder | None = None  # Global singleton instance of the system builder
    repr_ident: int = None  # Indentation level for string representation
    id_slice: slice = slice(-6, -1)  # Slice for identifiers
    with_py_loc: bool = False  # Whether to include Python location in string representation
    all_dirs_to_exclude: list = []  # Directories to exclude for stack inspection

    @classmethod
    def set_builder(mcs, builder: SysBuilder | None) -> None:
        '''Set or clear the global builder, preventing double registration.'''
        if builder is not None:
            if mcs._builder is not None:
                raise RuntimeError('Singleton builder already initialised')
            mcs._builder = builder
            return
        mcs._builder = None

    @classmethod
    def peek_builder(mcs) -> SysBuilder:
        '''Return the active builder, raising if none is registered.'''
        builder = mcs._builder
        if builder is None:
            raise RuntimeError('Singleton builder is not initialised')
        return builder

    @classmethod
    def initialize_dirs_to_exclude(mcs):
        '''Initialize the directories to exclude if not already initialized.'''
        if not mcs.all_dirs_to_exclude:
            site_package_dirs = site.getsitepackages()
            user_site_package_dir = site.getusersitepackages()
            mcs.all_dirs_to_exclude = site_package_dirs + [user_site_package_dir]
