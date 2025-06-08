'''The module provides the Array class for representing register arrays in the IR.'''

from __future__ import annotations

import typing

from ..builder import ir_builder, Singleton
from .dtype import to_uint, RecordValue, Record
from .expr import ArrayRead, ArrayWrite, Expr
from .value import Value
from ..utils import identifierize

if typing.TYPE_CHECKING:
    from .dtype import DType

def RegArray( #pylint: disable=invalid-name,too-many-arguments
        scalar_ty: DType,
        size: int,
        initializer: list = None,
        name: str = None,
        partition: str = None,
        attr: list = None,):
    '''
    The frontend API to declare a register array.

    Args:
        scalar_ty: The data type of the array elements.
        size: The size of the array. MUST be a compilation time constant.
        attr: The attribute list of the array.
        initializer: The initializer of the register array. If not set, it is 0-initialized.
    '''

    attr = attr if attr is not None else []

    if Array.FULLY_PARTITIONED in attr:
        partition = 'full'

    res = Array(scalar_ty, size, initializer, partition)
    if name is not None:
        res.name = name

    Singleton.builder.arrays.append(res)

    return res

class Array:  #pylint: disable=too-many-instance-attributes
    '''The class represents a register array in the AST IR.'''

    scalar_ty: DType  # Data type of each element in the array
    size: int  # Size of the array
    initializer: list  # Initial values for the array elements
    attr: list  # Attributes of the array
    parent: typing.Optional[Array]  # Parent array for partitioned arrays
    _users: typing.List[Expr]  # Users of the array
    _name: str  # Internal name storage
    _partition: list # Partitioned arrays

    FULLY_PARTITIONED = 1

    def as_operand(self):
        '''Dump the array as an operand.'''
        return self.name

    @property
    def name(self):
        '''The name of the array. If not set, a default name is generated.'''
        prefix = self._name if self._name is not None else 'array'
        return f'{prefix}_{identifierize(self)}'

    @name.setter
    def name(self, name):
        self._name = name

    @property
    def partition(self):
        '''Return the partitioned arrays or just this array if not partitioned.'''
        return self._partition if self._partition is not None else [self]

    def __init__(self, scalar_ty: DType, size: int, initializer: list, partition: str):
        #pylint: disable=import-outside-toplevel
        from .dtype import DType
        assert isinstance(scalar_ty, DType)
        self.scalar_ty = scalar_ty
        self.size = size
        self.initializer = initializer
        self.parent = None
        self.attr = []
        self._name = None
        self._partition = None
        self._users = []
        if partition == 'full':
            self._partition = []
            self.attr = [self.FULLY_PARTITIONED]
            for i in range(size):
                init = initializer[i] if initializer is not None else 0
                self._partition.append(Array(scalar_ty, 1, [init], None))
                self._partition[i].name = f'{self.name}_{i}'
                self._partition[i].parent = self

    @property
    def users(self):
        '''Get the users of the array.'''
        return self._users

    def __repr__(self):
        res = f'array {self.name}[{self.scalar_ty}; {self.size}] ='
        if self._partition is None:
            return res + f' {self.initializer}'
        res += ' [ '
        for i in range(self.size):
            res += f'{self._partition[i].name}, '
        return res + ' ]'

    @property
    def index_bits(self):
        '''Get the number of bits needed to index the array.'''
        is_p2 = self.size & (self.size - 1) == 0
        return self.size.bit_length() - is_p2

    def index_type(self):
        '''Get the type of the index.'''
        #pylint: disable=import-outside-toplevel
        from .dtype import UInt
        return UInt(self.index_bits)

    @ir_builder
    def __getitem__(self, index: typing.Union[int, Value]):
        res = None
        # If not partitioned, return the value at the given index
        if self._partition is None:
            if isinstance(index, int):
                index = to_uint(index, self.index_bits)
            res = ArrayRead(self, index)
        # If partitioned, return the value from the partitioned array
        # If the index is an integer, return the value from the partitioned array
        elif isinstance(index, int):
            res = self._partition[index][0]
        else:
            cases = { None: self._partition[0].__getitem__(0) }
            for i in range(self.size):
                cases[to_uint(i, self.index_bits)] = self._partition[i].__getitem__(0)
            res = index.case(cases)
            if isinstance(self.scalar_ty, Record):
                res = RecordValue(self.scalar_ty, res)
        assert res is not None, f'{res} is None'

        return res

    def get_flattened_size(self):
        '''Get the flattened size of the array.'''
        return self.size * self.scalar_ty.bits

    @ir_builder
    def __setitem__(self, index, value):

        if self._partition is None:
            if isinstance(index, int):
                index = to_uint(index)
            assert isinstance(index, Value)
            assert isinstance(value, (Value, RecordValue)), type(value)
            return ArrayWrite(self, index, value)

        # If the index is an integer, set the value in the partitioned array
        if isinstance(index, int):
            self._partition[index].__setitem__(0, value)
            return None

        # These imports need to be here to avoid circular imports
        from .block import Condition  # pylint: disable=import-outside-toplevel
        from .dtype import UInt  # pylint: disable=import-outside-toplevel
        idx_ty = UInt(self.index_bits)
        for i in range(self.size):
            with Condition(index.bitcast(idx_ty) == to_uint(i, self.index_bits)):
                self._partition[i].__setitem__(0, value)

        return None
