'''The module for the block AST node related implementations.'''

from __future__ import annotations

import typing

from ..builder import ir_builder, Singleton
from ..utils import identifierize, namify

if typing.TYPE_CHECKING:
    from .module.base import ModuleBase
    from .value import Value
    from .expr import Expr

class Block:
    '''The base node of a block.'''

    kind: int  # Kind of block
    _body: list[Expr]  # List of instructions in the block
    parent: typing.Union[typing.Self, ModuleBase]  # Parent block
    module: typing.Optional[ModuleBase]  # Module of this block

    MODULE_ROOT = 0
    CONDITIONAL = 1

    def __init__(self, kind: int):
        self.kind = kind
        self._body = []
        self.parent = self.module = None

    def __repr__(self):
        # Render block body respecting the current indent. Intrinsics control indent themselves.
        Singleton.repr_ident += 2
        lines = []
        for elem in self.iter():
            payload = repr(elem)
            lines.append((' ' * Singleton.repr_ident) + payload)
        Singleton.repr_ident -= 2
        return '\n'.join(lines)

    @property
    def body(self):
        '''Get the body of the block.'''
        return self._body

    def as_operand(self):
        '''Dump the block as an operand.'''
        return f'_{namify(identifierize(self))}'

    def insert(self, x, elem):
        '''Insert an instruction at the specified position.'''
        self._body.insert(x, elem)

    def iter(self):
        '''Iterate over the block.'''
        yield from self._body

    def __enter__(self):
        '''Designate the scope of entering the block.'''
        parent = Singleton.builder.current_block
        if parent is None:
            parent = Singleton.builder.current_module
        assert parent is not None
        self.parent = parent
        self.module = Singleton.builder.current_module
        Singleton.builder.enter_context_of('block', self)
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        '''Designate the scope of exiting the block.'''
        Singleton.builder.exit_context_of('block')

class CondBlock(Block):
    '''The inherited class of the block for conditional block.'''

    cond: Value  # Condition for this block

    def __init__(self, cond):
        super().__init__(Block.CONDITIONAL)
        # pylint: disable=import-outside-toplevel
        from .expr import Operand, Expr
        self.cond = Operand(cond, self)
        if isinstance(cond, Expr):
            cond.users.append(self.cond)

    def __repr__(self):
        ident = Singleton.repr_ident * ' '
        res = f'when {self.cond.as_operand()} {{\n'
        res = res + super().__repr__()
        res = res + f'\n{ident}}}'
        return res


class _PredicateScope:  # pylint: disable=too-few-public-methods
    '''Lightweight context manager that pushes/pops predicate intrinsics.'''

    def __init__(self, cond):
        self._cond = cond

    def __enter__(self):
        # pylint: disable=import-outside-toplevel
        from .expr.intrinsic import push_condition
        push_condition(self._cond)
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        # pylint: disable=import-outside-toplevel
        from .expr.intrinsic import pop_condition
        pop_condition()


def Condition(cond): # pylint: disable=invalid-name
    #pylint: disable=import-outside-toplevel
    '''Frontend API for conditionally guarding statements using predicate intrinsics.'''
    from .value import Value
    assert isinstance(cond, Value)
    return _PredicateScope(cond)

def Cycle(cycle: int): # pylint: disable=invalid-name
    '''Frontend helper returning a Condition sugar that checks current_cycle equals the given cycle.'''
    assert isinstance(cycle, int)
    # pylint: disable=import-outside-toplevel
    from .expr.intrinsic import current_cycle
    from .dtype import UInt
    return Condition(current_cycle() == UInt(64)(cycle))
