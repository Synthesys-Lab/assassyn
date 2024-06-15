from .value import Value

class Const(Value):
    def __init__(self, dtype, value):
        self.dtype = dtype
        self.value = value

    def __repr__(self):
        return f'({self.value}:{self.dtype})'

    def as_operand(self):
        return self.__repr__()
