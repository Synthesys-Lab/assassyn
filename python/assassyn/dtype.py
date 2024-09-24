'''Data type module for assassyn frontend'''

from .builder import ir_builder

#pylint: disable=too-few-public-methods,useless-parent-delegation

class DType:
    '''Base class for data type'''

    def __init__(self, bits: int):
        '''The constructor, only records the bits'''
        self.bits = bits

    def attributize(self, value):
        '''The syntax sugar for creating a port'''
        pass

class Void(DType):
    '''Void data type'''

    def __init__(self):
        super().__init__(1)

_VOID = Void()

def void():
    '''The syntax sugar for creating a void data type'''
    return _VOID

def _const_impl(dtype, value: int):
    '''The syntax sugar for creating a constant'''
    #pylint: disable=import-outside-toplevel
    from .const import Const
    return Const(dtype, value)


class Int(DType):
    '''Signed integer data type'''

    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'i{self.bits}'

    def __call__(self, value: int):
        return _const_impl(self, value)

class UInt(DType):
    '''Un-signed integer data type'''

    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'u{self.bits}'

    def __call__(self, value: int):
        return _const_impl(self, value)

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

    def __call__(self, value: int):
        return _const_impl(self, value)

class Record(DType):
    '''Record data type'''

    def __init__(self, **kwargs):
        '''Instantiate a record type with fields in kwargs.
        NOTE: After Python-3.6, the order of fields is guaranteed to be the same as the order fed to
        the argument. Thus, we can make the asumption that the order of feeding the arguments 
        is from msb to lsb.
        '''
        self.bits = 0
        self.fields = {}

        for name, dtype in kwargs.items():
            assert isinstance(dtype, DType)
            self.fields[name] = (dtype, slice(self.bits, self.bits + dtype.bits - 1))
            self.bits += dtype.bits

    def __repr__(self):
        fields = list('{name}: {dtype}' for name, (dtype, _) in self.fields.items())
        fields = ', '.join(fields)
        return f'record {{ {fields} }}'

    def bundle(self, **kwargs):
        '''The syntax sugar for creating a record'''
        #pylint: disable=import-outside-toplevel
        from .value import RecordValue
        print(self)
        return RecordValue(self, **kwargs)

    def __repr__(self):
        fields = list(f'{name}: {dtype}' for name, (dtype, _) in self.fields.items())
        fields = ', '.join(fields)
        return f'record {{ {fields} }}'

    def attributize(self, value, name):
        '''The reflective function for creating corresponding attributes of the host value'''
        assert name in self.fields, f'Field {name} not found in {self.fields} of this Record'
        dtype, attr_slice = self.fields[name]
        res = value[attr_slice]
        # TODO(@were): Handle more cases later.
        if not isinstance(dtype, Bits):
            res = res.bitcast(dtype)
        return res


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
