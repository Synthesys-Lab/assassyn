from .builder import ir_builder, Singleton
from .dtype import DType
from .expr import Expr

class Array(object):

    def __init__(self, scalar_ty: DType, size: int):
        self.scalar_ty = scalar_ty
        self.size = size
        Singleton.builder.arrays.append(self)

    @ir_builder(node_type='expr')
    def __getitem__(self, index):
        return Expr(Expr.ARRAY_READ, self, index)

    @ir_builder(node_type='expr')
    def __setitem__(self, index, value):
        return Expr(Expr.ARRAY_WRITE, self, index, value)

