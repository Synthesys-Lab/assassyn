class DType(object):
    def __init__(self, bits: int):
        pass

class Int(DType):
    def __init__(self, bits: int):
        super(self).__init__(bits)

class UInt(DType):
    def __init__(self, bits: int):
        super(self).__init__(bits)

class Float(DType):
    def __init__(self, bits: int):
        super(self).__init__(bits)
