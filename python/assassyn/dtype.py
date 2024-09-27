'''Data type module for assassyn frontend'''

#pylint: disable=too-few-public-methods,useless-parent-delegation,cyclic-import

class DType:
    '''Base class for data type'''

    def __init__(self, bits: int):
        '''The constructor, only records the bits'''
        self.bits = bits

    def attributize(self, value, name):
        '''The syntax sugar for creating a port'''

class Void(DType):
    '''Void data type'''

    def __init__(self):
        super().__init__(1)

_VOID = Void()

def void():
    '''The syntax sugar for creating a void data type'''
    return _VOID

class Int(DType):
    '''Signed integer data type'''

    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'i{self.bits}'

    def __call__(self, value: int):
        #pylint: disable=import-outside-toplevel
        from .const import _const_impl
        return _const_impl(self, value)

class UInt(DType):
    '''Un-signed integer data type'''

    def __init__(self, bits: int):
        super().__init__(bits)

    def __repr__(self):
        return f'u{self.bits}'

    def __call__(self, value: int):
        #pylint: disable=import-outside-toplevel
        from .const import _const_impl
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
        #pylint: disable=import-outside-toplevel
        from .const import _const_impl
        return _const_impl(self, value)

class Record(DType):
    '''Record data type'''

    def __init__(self, *args, **kwargs):
        '''Instantiate a record type with fields in kwargs.
        NOTE: After Python-3.6, the order of fields is guaranteed to be the same as the order fed to
        the argument. Thus, we can make the asumption that the order of feeding the arguments 
        is from msb to lsb.

        Args:
        *args: A dictionary of fields { (start, end): (name, dtype1) }
        **kwargs: A dictionary of fields { name: dtype2 }

        These two arguments are mutually exclusive.
        NOTE: dtype1 is the class of dtype or instance of dtype,
        while dtype2 is the instance of dtype.
        '''

        bits = 0
        self.fields = {}

        if args:
            assert len(args) == 1, "Expecting only one argument!"
            assert isinstance(args[0], dict), "Expecting a dictionary!"
            assert not kwargs, "Expecting no keyword arguments!"
            fields = args[0]
            for (start, end), (name, dtype) in fields.items():
                assert isinstance(start, int) and isinstance(end, int)
                assert 0 <= start <= end
                bitwidth = end - start + 1
                if dtype in [Int, UInt, Bits]:
                    dtype = dtype(bitwidth)
                elif isinstance(dtype, DType):
                    assert dtype.bits == bitwidth, f'Expecting {bitwidth} bits for {dtype}'
                else:
                    assert False, f'{dtype} cannot be constructed in Record'
                self.fields[name] = (dtype, slice(bits, bits + bitwidth - 1))
                bits += bitwidth
        elif kwargs:
            for name, dtype in reversed(kwargs.items()):
                assert isinstance(dtype, DType)
                self.fields[name] = (dtype, slice(bits, bits + dtype.bits - 1))
                bits += dtype.bits
        else:
            assert False, 'No fields provided for Record'

        super().__init__(bits)

    def bundle(self, **kwargs):
        '''The syntax sugar for creating a record'''
        #pylint: disable=import-outside-toplevel
        return RecordValue(self, **kwargs)

    def view(self, value):
        '''Create a view of RecordValue for the given value. For now, users should ensure the
        bits value has the same length as the record type.
        '''
        return RecordValue(self, value)

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


class RecordValue:
    '''The value class for the record type. Remember, this is a right-value object, so each
    field of this record is immutable!'''

    def __init__(self, dtype, *args, **kwargs):

        if args:
            assert len(args) == 1, "Expecting only one argument!"
            # TODO(@were): Strictly check the dtype
            # assert args[0].dtype == dtype, "Expecting the same Record type!"
            self.payload = args[0]
            self.dtype = dtype
            return

        assert isinstance(dtype, Record), "Expecting a Record type!"

        ordered_values = []
        fields_set = set(dtype.fields.keys())
        # TODO(@were): Check all the values are already in bits type
        for name, value in kwargs.items():
            assert name in dtype.fields, f'Field {name} not found in {dtype.fields} of this Record'
            fields_set.remove(name)
            _, attr_slice = dtype.fields[name]
            ordered_values.append((attr_slice, value))

        assert not fields_set, f'Fields are not fully initialized, missing: {fields_set}'
        ordered_values.sort(key=lambda x: -x[0].start)

        #pylint: disable=import-outside-toplevel
        from .expr import concat
        payload = concat(*[v for _, v in ordered_values])

        self.payload = payload
        self.dtype = dtype

    def as_operand(self):
        '''Return the payload as an operand'''
        return self.payload.as_operand()

    def __repr__(self):
        return f'RecordValue({self.dtype}, {self.payload})'

    # A Python TIP: __getattr__ is a "fallback" method, when "name" attribute is not found in the
    # self object. However, __getattribute__ is a "hook" method, which is called when every a.b
    # field access is made. If you do anything like self.a in __getattribute__, it will cause a
    # infinite recursion.
    def __getattr__(self, name):
        return self.dtype.attributize(self.payload, name)
