from .builder import ir_builder, Singleton
from .dtype import DType, UInt, to_uint
from .expr import Expr, BinaryOp, ArrayRead, ArrayWrite
from .value import Value

@ir_builder(node_type='array')
def RegArray(scalar_ty: DType, size: int, init_val=None):
    return Array(scalar_ty, size, init_val)

class Array(object):

    def as_operand(self):
        return self.name

    @property
    def name(self):
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

