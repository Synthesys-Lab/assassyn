#pylint: disable=too-few-public-methods,useless-parent-delegation

'''Data type module for assassyn frontend'''

class DType:
    '''Base class for data type'''

    def __init__(self, bits: int):
        '''The constructor, only records the bits'''
        self.bits = bits

    def __call__(self, value: int):
        '''The syntax sugar for creating a constant'''
        #pylint: disable=import-outside-toplevel
        from .const import Const
        return Const(self, value)

class Int(DType):
    '''Signed integer data type'''

    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'i{self.bits}'

class UInt(DType):
    '''Un-signed integer data type'''

    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'u{self.bits}'

class Float(DType):
    '''Floating point data type'''

    def __init__(self):
        super().__init__(32)

    def __repr__(self):
        return 'f32'

class Bits(DType):
    '''Raw bits data type'''

    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'b{self.bits}'

def to_uint(value: int, bits=None):
    '''
    Convert an integer to an unsigned integer constant with minimized bits

    Args:
        value: The integer value
        bits: The number of bits to use, default to the minimal bits needed
    '''
    assert isinstance(value, int)
    if bits is None:
        bits = max(value.bit_length(), 1)
    return UInt(bits)(value)

def to_int(value: int, bits=None):
    '''
    Convert an integer to a signed integer constant with minimized bits

    Args:
        value: The integer value
        bits: The number of bits to use, default to the minimal bits needed
    '''
    assert isinstance(value, int)
    if bits is None:
        bits = max(value.bit_length(), 1)
    return Int(bits)(value)
