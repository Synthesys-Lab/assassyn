'''The AST node module for constant values.'''

from .value import Value
from .dtype import Bits, DType
from ..utils.enforce_type import enforce_type

class Const(Value):
    '''The AST node data structure for constant values.'''

    dtype: DType  # Data type of this constant
    value: int  # The actual value of this constant

    @enforce_type
    def __init__(self, dtype: DType, value: int):
        assert dtype.inrange(value), f"Value {value} is out of range for {dtype}"
        self._dtype = dtype
        self.value = value

    @property
    def dtype(self):
        '''Get the data type of this constant'''
        return self._dtype

    @dtype.setter
    def dtype(self, value):
        '''Set the data type of this constant'''
        self._dtype = value

    def __repr__(self):
        return f'({self.value}:{self.dtype})'

    def as_operand(self):
        '''Dump the constant as an operand.'''
        return repr(self)

    @enforce_type
    def __getitem__(self, x: slice) -> 'Const':
        '''Override the value slicing operation.
        
        Note: Currently limited to 32 bits due to implementation constraints.
        This is a known limitation documented in Phase 2.
        '''
        bits = x.stop - x.start + 1
        assert 0 < bits <= 32, "TODO: Support more than 32 bits later"
        assert self.dtype.bits >= bits, f"Got {self.dtype.bits} bits, but {bits} bits are needed"
        dtype = Bits(bits)
        return _const_impl(dtype, (self.value >> x.start) & ((1 << bits) - 1))

    def concat(self, other):
        '''Concatenate two values together.'''

        if isinstance(other, Const):
            shift = other.dtype.bits
            return Bits(shift + self.dtype.bits)((self.value << shift) | other.value)

        return super().concat(other)


def _const_impl(dtype, value: int):
    '''The syntax sugar for creating a constant'''
    #pylint: disable=import-outside-toplevel
    from ..builder import Singleton

    builder = getattr(Singleton, 'builder', None)
    cache_key = None
    if builder is not None:
        cache = getattr(builder, 'const_cache', None)
        if cache is None:
            builder.const_cache = {}
            cache = builder.const_cache
        cache_key = (dtype, value)
        cached = cache.get(cache_key)
        if cached is not None:
            return cached

    const = Const(dtype, value)

    if builder is not None and cache_key is not None:
        builder.const_cache[cache_key] = const

    return const
