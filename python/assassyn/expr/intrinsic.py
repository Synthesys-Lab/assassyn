'''The module for intrinsic expressions'''
#pylint: disable=cyclic-import

from ..builder import ir_builder
from .expr import Expr

INTRIN_INFO = {
    # Intrinsic operations opcode: (mnemonic, num of args, side effect)
    900: ('wait_until', 1, True)
}

class Intrinsic(Expr):
    '''The class for intrinsic operations'''

    WAIT_UNTIL = 900

    def __init__(self, opcode, *args):
        super().__init__(opcode)
        _, num_args, _ = INTRIN_INFO[opcode]
        if num_args is not None:
            assert len(args) == num_args
        self.args = args

    def __repr__(self):
        args = {", ".join(i.as_operand() for i in self.args[0:])}
        mn, _, side_effect = INTRIN_INFO[self.opcode]
        side_effect = ['', 'side effect '][side_effect]
        return f'{self.as_operand()} = {side_effect}intrinsic.{mn}({args})'

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        return False

@ir_builder(node_type='expr')
def _wait_until(cond): #pylint: disable=invalid-name
    '''Frontend API for creating a wait-until block.'''
    #pylint: disable=import-outside-toplevel
    from ..value import Value
    assert isinstance(cond, Value)
    return Intrinsic(Intrinsic.WAIT_UNTIL, cond)
