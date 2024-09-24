'''The AST node module for constant values.'''

from .value import Value

class Const(Value):
    '''The AST node data structure for constant values.'''

    def __init__(self, dtype, value):
        self.dtype = dtype
        self.value = value

    def __repr__(self):
        return f'({self.value}:{self.dtype})'

    def as_operand(self):
        '''Dump the constant as an operand.'''
        return repr(self)

def _const_impl(dtype, value: int):
    '''The syntax sugar for creating a constant'''
    #pylint: disable=import-outside-toplevel
    return Const(dtype, value)
