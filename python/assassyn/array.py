'''The module provides the Array class for representing register arrays in the IR.'''

from .builder import ir_builder
from .dtype import DType, to_uint
from .expr import ArrayRead, ArrayWrite
from .value import Value

@ir_builder(node_type='array')
def RegArray(scalar_ty: DType, size: int): #pylint: disable=invalid-name
    '''
    The frontend API to declare a register array.

    Args:
        scalar_ty: The data type of the array elements.
        size: The size of the array. MUST be a compilation time constant.
    '''
    return Array(scalar_ty, size)

class Array:
    '''The class represents a register array in the AST IR.'''

    def as_operand(self):
        '''Dump the array as an operand.'''
        return self.name

    @property
    def name(self):
        '''The name of the array. If not set, a default name is generated.'''
        if self._name is not None:
            return self._name
        return f'array_{id(self)}'

    @name.setter
    def name(self, name):
        self._name = name

    def __init__(self, scalar_ty: DType, size: int):
        self.scalar_ty = scalar_ty
        self.size = size
        self._name = None

    def __repr__(self):
        return f'array {self.name}[{self.scalar_ty}; {self.size}]'

    @ir_builder(node_type='expr')
    def __getitem__(self, index):
        if isinstance(index, int):
            index = to_uint(index)
        assert isinstance(index, Value)
        return ArrayRead(self, index)

    @ir_builder(node_type='expr')
    def __setitem__(self, index, value):
        if isinstance(index, int):
            index = to_uint(index)
        assert isinstance(index, Value)
        return ArrayWrite(self, index, value)
