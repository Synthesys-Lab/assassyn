from .expr import Expr

class Const(Expr):
    def __init__(self, dtype, value):
        self.dtype = dtype
        self.value = value

    def __repr__(self):
        return f'({self.value}:{self.dtype})'

    def as_operand(self):
        return self.__repr__()

class DType(object):
    def __init__(self, bits: int):
        self.bits = bits

    def __call__(self, value: int):
        return Const(self, value)

class Int(DType):
    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'i{self.bits}'

class UInt(DType):
    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'u{self.bits}'

class Float(DType):
    def __init__(self):
        super().__init__(32)

    def __repr__(self):
        return f'f32'

class Bits(DType):
    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'b{self.bits}'

def to_uint(value, bits=None):
    if bits is None:
        bits = value.bit_length()
    return UInt(bits)(value)

def to_int(value, bits=None):
    if bits is None:
        bits = value.bit_length()
    return Int(bits)(value)
