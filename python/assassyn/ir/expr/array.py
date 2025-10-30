'''Array operations for expressions'''

#pylint: disable=cyclic-import

from __future__ import annotations

import typing

from ..value import Value
from .expr import Expr
from ...utils.enforce_type import enforce_type

if typing.TYPE_CHECKING:
    from ..array import Array
    from ..dtype import DType
    from ..module.base import ModuleBase


class ArrayWrite(Expr):
    '''The class for array write operation, where arr[idx] = val'''

    ARRAY_WRITE = 401

    @enforce_type
    # pylint: disable=too-many-arguments
    def __init__(
        self,
        arr: Array,
        idx: Value,
        val: Value,
        module: ModuleBase = None,
        meta_cond: typing.Optional[Value] = None,
    ):
        # Get module from Singleton if not provided
        if module is None:
            # pylint: disable=import-outside-toplevel
            from ...builder import Singleton
            module = Singleton.peek_builder().current_module
        super().__init__(ArrayWrite.ARRAY_WRITE, [arr, idx, val], meta_cond=meta_cond)
        self.module = module

    @property
    def array(self) -> Array:
        '''Get the array to write to'''
        return self._operands[0]

    @property
    def idx(self) -> Value:
        '''Get the index to write at'''
        return self._operands[1]

    @property
    def val(self) -> Value:
        '''Get the value to write'''
        return self._operands[2]

    @property
    def dtype(self):
        '''Get the data type of this operation (Void for side-effect operations)'''
        #pylint: disable=import-outside-toplevel
        from ..dtype import void
        return void()

    def __repr__(self):
        module_info = f' /* {self.module.name} */' if self.module else ''
        meta = self.meta_cond
        if meta is None:
            meta_info = ''
        else:
            operand = meta.as_operand() if hasattr(meta, 'as_operand') else repr(meta)
            meta_info = f' // meta cond {operand}'
        return (
            f'{self.array.as_operand()}[{self.idx.as_operand()}]'
            f' <= {self.val.as_operand()}{module_info}{meta_info}'
        )


class ArrayRead(Expr):
    '''The class for array read operation, where arr[idx] as a right value'''

    ARRAY_READ = 400

    @enforce_type
    def __init__(self, arr: Array, idx: Value):
        # pylint: disable=import-outside-toplevel
        from ..array import Array
        assert isinstance(arr, Array), f'{type(arr)} is not an Array!'
        assert isinstance(idx, Value), f'{type(idx)} is not a Value!'
        super().__init__(ArrayRead.ARRAY_READ, [arr, idx])

    @property
    def array(self) -> Array:
        '''Get the array to read from'''
        return self._operands[0]

    @property
    def idx(self) -> Value:
        '''Get the index to read at'''
        return self._operands[1]

    @property
    def dtype(self) -> DType:
        '''Get the data type of the read value'''
        return self.array.scalar_ty

    def __repr__(self):
        return f'{self.as_operand()} = {self.array.as_operand()}[{self.idx.as_operand()}]'

    def __getattr__(self, name):
        return self.dtype.attributize(self, name)

    def __le__(self, value):
        '''
        Handle the <= operator for array writes.
        '''
        # pylint: disable=import-outside-toplevel
        from ...builder import Singleton
        from ..dtype import RecordValue

        assert isinstance(value, (Value, RecordValue)), \
            f"Value must be Value or RecordValue, got {type(value)}"

        current_module = Singleton.peek_builder().current_module

        write_port = self.array & current_module
        return write_port._create_write(self.idx.value, value)
