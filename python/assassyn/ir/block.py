'''Predicate helper context managers for conditional execution.'''

from __future__ import annotations

import typing

if typing.TYPE_CHECKING:
    from .value import Value


class _PredicateScope:  # pylint: disable=too-few-public-methods
    '''Lightweight context manager that emits predicate push/pop intrinsics.'''

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


def Condition(cond):  # pylint: disable=invalid-name
    # pylint: disable=import-outside-toplevel
    '''Frontend API for conditionally guarding statements using predicate intrinsics.'''
    from .value import Value
    assert isinstance(cond, Value)
    return _PredicateScope(cond)


def Cycle(cycle: int):  # pylint: disable=invalid-name
    # pylint: disable=line-too-long
    '''Frontend helper returning a Condition sugar that checks current_cycle equals the given cycle.'''
    assert isinstance(cycle, int)
    # pylint: disable=import-outside-toplevel
    from .expr.intrinsic import current_cycle
    from .dtype import UInt
    return Condition(current_cycle() == UInt(64)(cycle))
