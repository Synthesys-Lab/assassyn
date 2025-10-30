'''
The module for multi-port register array access.
It defines the WritePort class and the ArrayWrite expression.
To support the (array & module)[index] <= value syntax.
'''

from __future__ import annotations
import typing

from ...builder import ir_builder
from .array import ArrayWrite
from ..dtype import to_uint, RecordValue, Record, Bits
from ..value import Value

if typing.TYPE_CHECKING:
    from ..array import Array
    from ..module.base import ModuleBase

class WritePort:
    '''
    Created via the (array & module) syntax to enable multi-port writes.
    '''

    array: Array
    module: ModuleBase

    def __init__(self, array: Array, module: ModuleBase):
        '''
        Initialize a WritePort.

        Args:
            array: The register array to write to
            module: The module that owns this write port
        '''
        self.array = array
        self.module = module

        if not hasattr(array, '_write_ports'):
            array._write_ports = {}

        if module not in array._write_ports:
            array._write_ports[module] = self

    def __getitem__(self, index):
        '''
        Return a proxy object that will handle the <= assignment.
        '''
        return IndexedWritePort(self, index)

    def __setitem__(self, index, value):
        '''
        Handles the `(a&self)[0] = v` syntax directly.
        '''
        return self._create_write(index, value)


    def _create_write(self, index, value):
        '''
        Create an ArrayWrite operation with module information.
        Enforces strict type checking between the written value and array's scalar_ty.
        '''

        if isinstance(index, int):
            index = to_uint(index)
        assert isinstance(index, Value), f"Index must be a Value, got {type(index)}"
        assert isinstance(value, (Value, RecordValue)), \
            f"Value must be a Value or RecordValue, got {type(value)}"

        # Handle RecordValue: extract dtype and unwrap
        if isinstance(value, RecordValue):
            value_dtype = value.dtype  # Get Record type for checking
            value = value.value()  # Unwrap to raw Bits now
        else:
            value_dtype = value.dtype

        # Type check using the extracted dtype
        # Special case: if array expects Record type and value is raw Bits,
        # allow if bit widths match (following Bind._push pattern)
        if isinstance(self.array.scalar_ty, Record) and isinstance(value_dtype, Bits):
            if value_dtype.bits != self.array.scalar_ty.bits:
                raise TypeError(
                    f"Type mismatch in array write: array '{self.array.name}' "
                    f"expects element type {self.array.scalar_ty} "
                    f"({self.array.scalar_ty.bits} bits), "
                    f"but got value of type {value_dtype} ({value_dtype.bits} bits)"
                )
            # Allow the write - bit widths match
        elif not self.array.scalar_ty.type_eq(value_dtype):
            raise TypeError(
                f"Type mismatch in array write: array '{self.array.name}' "
                f"expects element type {self.array.scalar_ty}, "
                f"but got value of type {value_dtype}"
            )

        @ir_builder
        def create_write():
            # pylint: disable=import-outside-toplevel
            from .intrinsic import get_pred
            meta_cond = get_pred()
            return ArrayWrite(self.array, index, value, self.module, meta_cond=meta_cond)

        return create_write()

    def __repr__(self):
        return f'WritePort({self.array.name}, {self.module.name})'

# pylint: disable=too-few-public-methods
class IndexedWritePort:
    '''
    A proxy object returned by WritePort.__getitem__ to handle the <= assignment.
    '''
    write_port: WritePort
    index: typing.Union[int, Value]

    def __init__(self, write_port, index):
        self.write_port = write_port
        self.index = index

    def __le__(self, value):
        '''
        Overload <= operator for non-blocking assignment syntax.
        '''
        return self.write_port._create_write(self.index, value)
