'''The module provides the Array class for representing register arrays in the IR.'''

from __future__ import annotations

import typing

from ..builder import ir_builder, Singleton
from .dtype import to_uint, RecordValue
from .expr import ArrayRead, Expr,BinaryOp
from .value import Value
from ..utils import identifierize
from .writeport import WritePort

if typing.TYPE_CHECKING:
    from .dtype import DType
    from .module.base import ModuleBase



def RegArray( #pylint: disable=invalid-name,too-many-arguments
        scalar_ty: DType,
        size: int,
        initializer: list = None,
        name: str = None,
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


    res = Array(scalar_ty, size, initializer)
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
    _users: typing.List[Expr]  # Users of the array
    _name: str  # Internal name storage
    _write_ports: typing.Dict['ModuleBase', 'WritePort'] = {} # Write ports for this array


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

    def __init__(self, scalar_ty: DType, size: int, initializer: list):
        #pylint: disable=import-outside-toplevel
        from .dtype import DType
        assert isinstance(scalar_ty, DType)
        self.scalar_ty = scalar_ty
        self.size = size
        self.initializer = initializer
        self.attr = []
        self._name = None
        self._users = []
        self._write_ports = {}
    @property
    def users(self):
        '''Get the users of the array.'''
        return self._users

    def __and__(self, other):
        '''
        Overload & operator to create WritePort when combined with a Module.
        This enables write access: (array & module)[idx] <= value
        '''
        from .module.base import ModuleBase #pylint: disable=import-outside-toplevel
        if isinstance(other, ModuleBase):
            if other not in self._write_ports:
                self._write_ports[other] = WritePort(self, other)
            return self._write_ports[other]

        # Fall back to regular bitwise AND with Value
        if isinstance(other, Value):
            return BinaryOp(BinaryOp.BITWISE_AND, self, other)

        raise TypeError(f"Cannot AND Array with {type(other)}")

    def __repr__(self):
        '''Enhanced repr to show write port information'''
        res = f'array {self.name}[{self.scalar_ty}; {self.size}] ='

        # Add write port information if any
        if hasattr(self, '_write_ports') and self._write_ports:
            port_info = f' /* {len(self._write_ports)} write ports: '
            port_info += ', '.join(m.name for m in self._write_ports)
            port_info += ' */'
            res += port_info

        res += ' [ '
        res += '\n'
        res += f'{self.name}, '
        if self._write_ports:
            res += f'write_ports: /* {len(self._write_ports)}: '
            res += ', '.join(m.name for m in self._write_ports)
            res += ' */'

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

    def get_write_ports(self):
        '''Get the write_ports.'''
        return getattr(self, '_write_ports', {})

    @ir_builder
    def __getitem__(self, index: typing.Union[int, Value]):
        if isinstance(index, int):
            index = to_uint(index, self.index_bits)
        res = ArrayRead(self, index)
        return res

    def get_flattened_size(self):
        '''Get the flattened size of the array.'''
        return self.size * self.scalar_ty.bits

    @ir_builder
    def __setitem__(self, index, value):

        if isinstance(index, int):
            index = to_uint(index)
        assert isinstance(index, Value)
        assert isinstance(value, (Value, RecordValue)), type(value)
        current_module = Singleton.builder.current_module
        write_port = self & current_module
        return write_port._create_write(index, value)
