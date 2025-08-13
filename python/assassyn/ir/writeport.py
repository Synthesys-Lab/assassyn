'''
The module for multi-port register array access.
It defines the WritePort class and the MultiPortArrayWrite expression.
To support the (array & module)[index] <= value syntax.
'''

from __future__ import annotations
import typing

from ..builder import ir_builder
from .expr import ArrayWrite
from .dtype import to_uint, RecordValue
from .value import Value

if typing.TYPE_CHECKING:
    from .array import Array
    from .module.base import ModuleBase

class WritePort:
    '''
    Created via the (array & module) syntax to enable multi-port writes.
    '''

    array: 'Array'
    module: 'ModuleBase'

    def __init__(self, array: 'Array', module: 'ModuleBase'):
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
        '''

        if isinstance(index, int):
            index = to_uint(index)
        assert isinstance(index, Value), f"Index must be a Value, got {type(index)}"
        assert isinstance(value, (Value, RecordValue)), \
            f"Value must be a Value or RecordValue, got {type(value)}"

        @ir_builder
        def create_write():
            return MultiPortArrayWrite(self.array, index, value, self.module)

        return create_write()

    def __repr__(self):
        return f'WritePort({self.array.name}, {self.module.name})'

# pylint: disable=too-few-public-methods
class IndexedWritePort:
    '''
    A proxy object returned by WritePort.__getitem__ to handle the <= assignment.
    '''
    write_port: 'WritePort'
    index: typing.Union[int, 'Value']

    def __init__(self, write_port, index):
        self.write_port = write_port
        self.index = index


    def __le__(self, other):
        '''
        Overload <= operator for non-blocking assignment syntax.
        '''
        value = other
        return self.write_port._create_write(self.index, value)

class PartitionedWritePort:
    '''
    A specialized WritePort for partitioned arrays.
    '''
    def __init__(self, array, module):

        assert array._partition is not None, "This must be used with a partitioned array."
        self.array = array
        self.module = module

    def __getitem__(self, index):
        if isinstance(index, int):
            # Get the correct sub-array and its corresponding WritePort
            sub_array = self.array._partition[index]
            write_port = WritePort(sub_array, self.module)
            return write_port.__getitem__(0) # write to index 0 of the single-element sub_array

        from .block import Condition  # pylint: disable=import-outside-toplevel
        from .dtype import UInt  # pylint: disable=import-outside-toplevel
        arr = self.array
        idx_ty = UInt(arr.index_bits)
        for i in range(arr.size):
            with Condition(index.bitcast(idx_ty) == to_uint(i, arr.index_bits)):
                # Get the correct sub-array and its corresponding WritePort
                sub_array = arr._partition[i]
                write_port = WritePort(sub_array, self.module)
                return write_port.__getitem__(0) # write to index 0 of the single-element sub_array

class MultiPortArrayWrite(ArrayWrite):
    '''
    Array write operation that tracks which module performs the write.
    This enables multiple modules to write to the same array in parallel.
    '''

    module: 'ModuleBase'

    def __init__(self, arr: 'Array', idx: 'Value', val: 'Value', module: 'ModuleBase'):
        '''
        Initialize a multi-port array write.

        Args:
            arr: The array being written to
            idx: The index to write at
            val: The value to write
            module: The module performing this write
        '''
        super().__init__(arr, idx, val)
        self.module = module

    def __repr__(self):
        module_info = f' /* {self.module.name} */' if self.module else ''
        return (
            f'{self.array.as_operand()}[{self.idx.as_operand()}]'
            f' <= {self.val.as_operand()}{module_info}'
        )
