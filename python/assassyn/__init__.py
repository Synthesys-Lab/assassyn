"""Assassyn's python frontend."""

class Module(object):
    def __init__(self):
        self.a = Port(DType())
        pass

class Port(object):
    def __init__(self, dtype: DType):
        pass

    @ir_builder
    def valid(self):
        pass

    @ir_builder
    def peek(self):
        pass

    @ir_builder
    def pop(self):
        pass

    @ir_builder
    def push(self):
        pass

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
