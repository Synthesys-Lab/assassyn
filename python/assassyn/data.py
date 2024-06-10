from .builder import ir_builder, Singleton
from .dtype import DType, UInt, to_uint
from .expr import Expr, BinaryOp, SideEffect

@ir_builder(node_type='array')
def RegArray(scalar_ty: DType, size: int):
    res = Array(scalar_ty, size)
    res.id = Singleton.builder.inc_id()
    return res

class Array(object):

    def as_operand(self):
        return self.name

    @property
    def name(self):
        return f'array_{self.id}'

    @property
    def id(self):
        return self._id

    @id.setter
    def id(self, val):
        self._id = val

    def __init__(self, scalar_ty: DType, size: int):
        self.scalar_ty = scalar_ty
        self.size = size
        self._id = None

    def __repr__(self):
        return f'array {self.name}[{self.scalar_ty}; {self.size}]'

    @ir_builder(node_type='expr')
    def __getitem__(self, index):
        if isinstance(index, int):
            index = to_uint(index)
        assert isinstance(index, Expr)
        return BinaryOp(BinaryOp.ARRAY_READ, self, index)

    @ir_builder(node_type='expr')
    def __setitem__(self, index, value):
        if isinstance(index, int):
            index = to_uint(index)
        assert isinstance(index, Expr)
        return SideEffect(SideEffect.ARRAY_WRITE, self, index, value)

