from .expr import Expr

class Const(Expr):
    def __init__(self, dtype, value):
        self.dtype = dtype
        self.value = value

    def __repr__(self):
        return f'({self.value}:{self.dtype})'

    def as_operand(self):
        return self.__repr__()
