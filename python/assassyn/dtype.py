class Const(object):
    def __init__(self, dtype, value):
        self.dtype = dtype
        self.value = value

class DType(object):
    def __init__(self, bits: int):
        self.bits = bits

    def __call__(self, value: int):
        return Const(self, value)

class Int(DType):
    def __init__(self, bits: int):
        super().__init__(bits)

class UInt(DType):
    def __init__(self, bits: int):
        super().__init__(bits)

class Float(DType):
    def __init__(self, bits: int):
        super().__init__(bits)
