'''The module for intrinsic expressions'''
#pylint: disable=cyclic-import

from ..builder import ir_builder
from .expr import Expr

INTRIN_INFO = {
    # Intrinsic operations opcode: (mnemonic, num of args, valued, side effect)
    900: ('wait_until', 1, False, True),
    901: ('finish', 0, False, True),
    902: ('assert', 1, False, True),
    903: ('barrier', 1, False, True),
}

class Intrinsic(Expr):
    '''The class for intrinsic operations'''

    WAIT_UNTIL = 900
    FINISH = 901
    ASSERT = 902
    BARRIER = 903

    opcode: int  # Operation code for this intrinsic
    args: tuple  # Arguments to the intrinsic operation

    def __init__(self, opcode, *args):
        super().__init__(opcode)
        _, num_args, _, _ = INTRIN_INFO[opcode]
        if num_args is not None:
            assert len(args) == num_args
        self.args = args

    def __repr__(self):
        args = {", ".join(i.as_operand() for i in self.args[0:])}
        mn, _, valued, side_effect = INTRIN_INFO[self.opcode]
        side_effect = ['', 'side effect '][side_effect]
        rhs = f'{side_effect}intrinsic.{mn}({args})'
        if valued:
            return f'{self.as_operand()} = {rhs}'
        return rhs

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        return False

@ir_builder
def wait_until(cond):
    '''Frontend API for creating a wait-until block.'''
    #pylint: disable=import-outside-toplevel
    from ..value import Value
    assert isinstance(cond, Value)
    return Intrinsic(Intrinsic.WAIT_UNTIL, cond)

@ir_builder
def assume(cond):
    '''Frontend API for creating an assertion.
    This name is to avoid conflict with the Python keyword.'''
    #pylint: disable=import-outside-toplevel
    from ..value import Value
    assert isinstance(cond, Value)
    return Intrinsic(Intrinsic.ASSERT, cond)


def is_wait_until(expr):
    '''Check if the expression is a wait-until intrinsic.'''
    return isinstance(expr, Intrinsic) and expr.opcode == Intrinsic.WAIT_UNTIL

@ir_builder
def finish():
    '''Finish the simulation.'''
    return Intrinsic(Intrinsic.FINISH)

@ir_builder
def barrier(node):
    '''Barrier the current simulation state.'''
    return Intrinsic(Intrinsic.BARRIER, node)
