'''The module for function-call related operations'''

from __future__ import annotations

import typing

from ...builder import ir_builder
from .expr import Expr

if typing.TYPE_CHECKING:
    from ..module import Module, Port

class FIFOPush(Expr):
    '''The class for FIFO push operation'''

    fifo: Port  # FIFO port to push to
    bind: Bind  # Bind reference
    fifo_depth: int  # Depth of the FIFO

    FIFO_PUSH  = 302

    def __init__(self, fifo, val):
        super().__init__(FIFOPush.FIFO_PUSH, [fifo, val])
        self.bind = None
        self.fifo_depth = None

    @property
    def fifo(self):
        '''Get the FIFO port'''
        return self._operands[0]

    @property
    def val(self):
        '''Get the value to push'''
        return self._operands[1]

    @property
    def dtype(self):
        '''Get the data type of this operation (Void for side-effect operations)'''
        #pylint: disable=import-outside-toplevel
        from ..dtype import void
        return void()

    def __repr__(self):
        handle = self.as_operand()
        return f'{self.fifo.as_operand()}.push({self.val.as_operand()}) // handle = {handle}'

class Bind(Expr):
    '''The class for binding operations. Function bind is a functional programming concept like
    Python's `functools.partial`.'''

    callee: Module  # Module being bound
    fifo_depths: dict  # Dictionary of FIFO depths
    pushes: list[FIFOPush] # List of FIFOPush operations

    BIND = 501

    def _push(self, **kwargs):
        #pylint: disable=import-outside-toplevel
        from ..dtype import RecordValue

        for k, v in kwargs.items():
            port = getattr(self.callee, k)

            # Handle RecordValue early: extract dtype and unwrap
            if isinstance(v, RecordValue):
                value_dtype = v.dtype  # Get Record type for checking
                v = v.value()  # Unwrap to raw Bits now
            else:
                value_dtype = v.dtype

            # Type check using the extracted dtype
            if not port.dtype.type_eq(value_dtype):
                raise ValueError(
                    f"Type mismatch in Bind: port '{k}' expects type {port.dtype}, "
                    f"but got value of type {value_dtype}"
                )

            # v is already unwrapped if it was RecordValue
            push = port.push(v)
            push.bind = self
            self.pushes.append(push)

    def bind(self, **kwargs):
        '''The exposed frontend function to instantiate a bind operation'''
        self._push(**kwargs)
        return self

    def is_fully_bound(self):
        '''The helper function to check if all the ports are bound.'''
        fifo_names = set(push.fifo.name for push in self.pushes)
        ports = self.callee.ports
        cnt = sum(i.name in fifo_names for i in ports)
        return cnt == len(ports)

    @property
    def pushes(self):
        '''Get the list of pushes'''
        return self._operands

    @ir_builder
    def async_called(self, **kwargs):
        '''The exposed frontend function to instantiate an async call operation'''
        self._push(**kwargs)
        return AsyncCall(self)

    def __init__(self, callee, **kwargs):
        super().__init__(Bind.BIND, [])
        self.callee = callee
        self._push(**kwargs)
        self.fifo_depths = {}

    def set_fifo_depth(self, **kwargs):
        """Set FIFO depths using keyword arguments."""
        for name, depth in kwargs.items():
            if not isinstance(depth, int):
                raise ValueError(f"Depth for {name} must be an integer")
            matches = 0
            available_fifos = []
            for push in self.pushes:
                available_fifos.append(push.fifo.name)
                if push.fifo.name == name:
                    push.fifo_depth = depth
                    matches = matches + 1
                    #break
            if matches == 0:
                raise ValueError(f"No push found for FIFO named {name}. "
                                 f"Available FIFO names are: {available_fifos}")


        return self

    @property
    def dtype(self):
        '''Get the data type of this operation (Void for binding operations)'''
        #pylint: disable=import-outside-toplevel
        from ..dtype import void
        return void()

    def __repr__(self):
        args = []
        for v in self.pushes:
            depth = self.fifo_depths.get(v.as_operand())
            depth_str = f", depth={depth}" if depth is not None else ""
            operand = v.as_operand()
            operand = f'{operand} /* {v.fifo.as_operand()}={v.val.as_operand()}{depth_str} */'
            args.append(operand)
        args = ', '.join(args)
        callee = self.callee.as_operand()
        lval = self.as_operand()
        fifo_depths_str = ', '\
            .join(f"{k}: {v}" for k, v in self.fifo_depths.items() if v is not None)
        fifo_depths_repr = f" /* fifo_depths={{{fifo_depths_str}}} */" if fifo_depths_str else ""
        return f'{lval} = {callee}.bind([{args}]){fifo_depths_repr}'

class AsyncCall(Expr):
    '''The class for async call operations. It is used to call a function asynchronously.'''

    # Call operations
    ASYNC_CALL = 500

    def __init__(self, bind: Bind):
        super().__init__(AsyncCall.ASYNC_CALL, [bind])
        bind.callee.users.append(self)

    @property
    def bind(self) -> Bind:
        '''Get the bind operation'''
        return self._operands[0]

    @property
    def dtype(self):
        '''Get the data type of this operation (Void for call operations)'''
        #pylint: disable=import-outside-toplevel
        from ..dtype import void
        return void()

    def __repr__(self):
        bind = self.bind.as_operand()
        return f'async_call {bind}'
