'''Data type module for assassyn frontend'''

from .value import Value
from .expr.comm import concat

#pylint: disable=too-few-public-methods,useless-parent-delegation,cyclic-import,unused-argument

class DType:
    '''Base class for data type'''

    _bits: int  # Number of bits in this data type

    def __init__(self, bits: int):
        '''The constructor, only records the bits'''
        self._bits = bits

    @property
    def bits(self):
        '''The number of bits in this data type'''
        return self._bits

    def __eq__(self, other):
        '''Check if two data types are equal'''
        return self.__class__ == other.__class__ and self.bits == other.bits

    def __hash__(self):
        '''Hash consistent with equality for caching purposes'''
        return hash((self.__class__, self.bits))

    def type_eq(self, other):
        '''Check if two data types are exactly equal.
        This is used for strict type checking in operations like Bind.
        By default, uses __eq__ which checks class and bits equality.
        '''
        return self == other

    def attributize(self, value, name):
        '''The syntax sugar for creating a port'''

    def inrange(self, value):
        '''Check if the value is in the range of the data type'''
        return True

    def is_int(self):
        '''Check if this is an integer data type'''
        return isinstance(self, (Int, UInt))

    def is_raw(self):
        '''Check if this is a raw bits data type'''
        return isinstance(self, Bits)

    def is_signed(self):
        '''Check if this is a signed data type'''
        return isinstance(self, Int)

class Void(DType):
    '''Void data type'''

    def __init__(self):
        super().__init__(1)

    def inrange(self, value):
        return False

class ArrayType(DType):

    '''Array data type'''

    def __init__(self, dtype, size):
        super().__init__(size * dtype.bits)
        self._scalar_ty = dtype
        self._size = size

    @property
    def size(self):
        '''The number of elements in this array'''
        return self._size

    @property
    def scalar_ty(self):
        '''The data type of the elements in this array'''
        return self._scalar_ty

    def type_eq(self, other):
        '''Check if two ArrayType types are exactly equal.'''
        if not isinstance(other, ArrayType):
            return False
        if self.size != other.size:
            return False
        return self.scalar_ty.type_eq(other.scalar_ty)


_VOID = Void()

def void():
    '''The syntax sugar for creating a void data type'''
    return _VOID

class Int(DType):
    '''Signed integer data type'''

    def __init__(self, bits: int):
        assert isinstance(bits, int), 'Expecting an integer for the bitwidth'
        super().__init__(bits)

    def __repr__(self):
        return f'i{self.bits}'

    def __call__(self, value: int):
        #pylint: disable=import-outside-toplevel
        from .const import _const_impl
        return _const_impl(self, value)

    def inrange(self, value):
        left = -(1 << (self.bits - 1))
        right = (1 << (self.bits - 1)) - 1
        return left <= value <= right

class UInt(DType):
    '''Un-signed integer data type'''

    def __init__(self, bits: int):
        assert isinstance(bits, int), 'Expecting an integer for the bitwidth'
        bits = max(bits, 1)
        super().__init__(bits)

    def __repr__(self):
        return f'u{self.bits}'

    def __call__(self, value: int):
        #pylint: disable=import-outside-toplevel
        from .const import _const_impl
        return _const_impl(self, value)

    def inrange(self, value):
        return 0 <= value < (1 << self.bits)

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

    def inrange(self, value):
        right = (1 << self.bits) - 1
        return 0 <= value <= right

class Record(DType):
    '''Record data type'''

    fields: dict  # Dictionary mapping field names to (dtype, slice) tuples
    readonly: bool  # Whether this record is readonly

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
                self.fields[name] = (dtype, slice(start, end))
                bits = max(bits, end+1)
            mask = [None] * bits
            for (start, end), (name, _) in fields.items():
                for i in range(start, end + 1):
                    assert mask[i] is None, f'Field {mask[i]} and {name} overlap'
                    mask[i] = name
            self.readonly = any(i is None for i in mask)
        elif kwargs:
            for name, dtype in reversed(kwargs.items()):
                assert isinstance(dtype, DType)
                self.fields[name] = (dtype, slice(bits, bits + dtype.bits - 1))
                bits += dtype.bits
            self.readonly = False
        else:
            assert False, 'No fields provided for Record'

        super().__init__(bits)

    def bundle(self, **kwargs):
        '''The syntax sugar for creating a record'''
        assert not self.readonly, 'Cannot bundle a readonly record'
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

    def type_eq(self, other):
        '''Check if two Record types are exactly equal by comparing structure.'''
        if not isinstance(other, Record) or self.bits != other.bits:
            return False
        if set(self.fields.keys()) != set(other.fields.keys()):
            return False
        for name, (dtype, slice_obj) in self.fields.items():
            if name not in other.fields:
                return False
            other_dtype, other_slice = other.fields[name]
            if not dtype.type_eq(other_dtype):
                return False
            if slice_obj.start != other_slice.start or slice_obj.stop != other_slice.stop:
                return False
        return True

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

    _payload: Value  # The underlying value of this record
    _dtype: Record  # The record type of this value

    def __init__(self, dtype, *args, **kwargs):

        if args:
            assert len(args) == 1, "Expecting only one argument!"
            # TODO(@were): Strictly check the dtype
            # assert args[0].dtype == dtype, "Expecting the same Record type!"
            self._payload = args[0]
            self._dtype = dtype
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

        payload = concat(*[v for _, v in ordered_values])

        self._payload = payload
        self._dtype = dtype

    def value(self):
        '''Return the payload as a value'''
        return self._payload

    def as_operand(self):
        '''Return the payload as an operand'''
        return self._payload.as_operand()

    @property
    def dtype(self):
        '''Return the Record type of this value.'''
        return self._dtype

    def __repr__(self):
        return f'RecordValue({self._dtype}, {self._payload})'

    def __call__(self, value):
        '''The syntax sugar for creating a record value'''
        return Bits(self._dtype.bits)(value)

    # A Python TIP: __getattr__ is a "fallback" method, when "name" attribute is not found in the
    # self object. However, __getattribute__ is a "hook" method, which is called when every a.b
    # field access is made. If you do anything like self.a in __getattribute__, it will cause a
    # infinite recursion.
    #
    # This is about an design decision in Python frontend: If you see this later, DO NOT try to
    # unify the `__getattr__` in `ArrayRead` and `FIFOPop` by creating an instance of `RecordValue`,
    # unless you read below and come up with a better design.
    #`RecordValue` is a virtual node which does not exist in the AST, the `ir_builder` decorator
    # can only push the generated node into the AST, which is the `ArrayRead` or `FIFOPop` node.
    # If you return a `RecordValue` that wraps these two nodes, this `RecordValue` will be pushed
    # into the AST, which is not what we want. Unless we can have a divergence in the returned
    # object and the wrapped object.
    def __getattr__(self, name):
        return self._dtype.attributize(self._payload, name)
