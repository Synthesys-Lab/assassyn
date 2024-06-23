'''The module provides the Array class for representing register arrays in the IR.'''

from .builder import ir_builder
from .dtype import DType, to_uint
from .expr import ArrayRead, ArrayWrite
from .value import Value

@ir_builder(node_type='array')
def RegArray(scalar_ty: DType, size: int, init_val=None): #pylint: disable=invalid-name
    '''
    The frontend API to declare a register array.

    Args:
        scalar_ty: The data type of the array elements.
        size: The size of the array. MUST be a compilation time constant.
        init_val: The value of the register array. If not set, the array is initialized as 0.
    '''
    return Array(scalar_ty, size, init_val)

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

    def __init__(self, scalar_ty: DType, size: int, init_val=None):
        self.scalar_ty = scalar_ty
        self.size = size
        self._name = None
        self.init_val = init_val

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
