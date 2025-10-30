'''The AST node data structure for the expressions'''

#pylint: disable=cyclic-import,import-outside-toplevel,too-many-instance-attributes

from __future__ import annotations

import typing

from ...builder import ir_builder
from ..value import Value
from ...utils import namify, identifierize

if typing.TYPE_CHECKING:
    from ..array import Array
    from ..module import Port, Module
    from ..module.base import ModuleBase
    from ..dtype import DType

class Operand:
    '''The base class for all operands. It is used to dump the operand as a string.'''
    _value: Value # The value of this operand
    _user: Expr # The user of this operand

    def __init__(self, value: Value, user: Expr):
        self._value = value
        self._user = user

    @property
    def value(self):
        '''Get the value of this operand'''
        return self._value

    @property
    def user(self):
        '''Get the user of this operand'''
        return self._user

    def __getattr__(self, name):
        '''Forward the attribute access to the value'''
        return getattr(self.value, name)

class Expr(Value):
    '''The frontend base node for expressions'''

    opcode: int  # Operation code for this expression
    loc: str  # Source location information
    parent: typing.Optional[ModuleBase]  # Parent module of this expression
    users: typing.List[Operand]  # List of users of this expression
    _operands: typing.List[
        typing.Union[Operand, Port, Array, int]
    ] # List of operands of this expression

    def __init__(self, opcode, operands: list, *, meta_cond: typing.Optional[Value] = None):
        '''Initialize the expression with an opcode'''
        #pylint: disable=import-outside-toplevel,too-many-locals
        self.opcode = opcode
        self.loc = self.parent = None
        self.name = None  # Initialize name attribute
        # NOTE: We only wrap values in Operand, not Ports or Arrays
        self._operands = []
        for operand in operands:
            self._operands.append(self._prepare_operand(operand))
        self.users = []
        override = self._normalize_meta_cond(meta_cond)
        if override is not None:
            self._meta_cond = override
        else:
            self._meta_cond = self._resolve_ambient_meta_cond()

    @staticmethod
    def _normalize_meta_cond(value: typing.Optional[Value]):
        return value.value if isinstance(value, Operand) else value

    def _resolve_ambient_meta_cond(self):
        try:
            # pylint: disable=import-outside-toplevel
            from ...builder import Singleton
            builder = Singleton.peek_builder()
        except (RuntimeError, ImportError):
            return None
        return self._normalize_meta_cond(builder.current_predicate_carry())

    def _prepare_operand(self, operand):
        '''Normalize an incoming operand and register its usage'''
        #pylint: disable=import-outside-toplevel
        from ..array import Array
        from ..const import Const
        from ..module import Port, Module
        from ..dtype import RecordValue
        from ...builder import Singleton
        from ..module.downstream import Downstream

        if isinstance(operand, (Array, Port)):
            operand.users.append(self)
            return operand

        if isinstance(operand, Expr):
            builder = Singleton.peek_builder()
            return self._prepare_expr_operand(operand, builder.current_module)

        if isinstance(operand, (Const, str, RecordValue, Module, Downstream)):
            return Operand(operand, self)

        raise AssertionError(f'{operand} is a {type(operand)}')

    def _prepare_expr_operand(self, expr_operand: Expr, current_module):
        '''Wrap an expression operand and enforce module ownership rules'''
        #pylint: disable=import-outside-toplevel
        from ..module.downstream import Downstream
        from .call import Bind

        if isinstance(expr_operand, Bind):
            wrapped = Operand(expr_operand, self)
            expr_operand.users.append(wrapped)
            return wrapped

        if not isinstance(current_module, Downstream):
            expr_module = expr_operand.parent if expr_operand.parent else None
            if not self._is_cross_module_allowed(expr_operand):
                assert current_module == expr_module, (
                    f'Expression {expr_operand} is from module {expr_module}, '
                    f'but current module is {current_module}'
                )

        wrapped = Operand(expr_operand, self)
        expr_operand.users.append(wrapped)
        return wrapped

    def _is_cross_module_allowed(self, expr_operand: Expr) -> bool:
        '''Check whether we allow cross-module usage for the given expression'''
        # Allow PureIntrinsic for external module output reads
        #pylint: disable=import-outside-toplevel
        from .intrinsic import PureIntrinsic, ExternalIntrinsic
        if isinstance(expr_operand, PureIntrinsic):
            if expr_operand.opcode == PureIntrinsic.EXTERNAL_OUTPUT_READ:
                return True

        # Allow ExternalIntrinsic to be used across modules
        if isinstance(expr_operand, ExternalIntrinsic):
            return True

        return False

    def get_operand(self, idx: int):
        '''Get the operand at the given index'''
        if idx < 0 or idx >= len(self._operands):
            raise IndexError(f'Index {idx} out of range for {self}')
        return self._operands[idx]

    @property
    def operands(self):
        '''Get the operands of this expression'''
        return self._operands

    @property
    def meta_cond(self):
        '''Return the cumulative predicate guarding this expression.'''
        return self._meta_cond


    def as_operand(self):
        '''Dump the expression as an operand'''
        # Use the name if assigned by the naming system
        if self.name is not None:
            return self.name
        return f'_{namify(identifierize(self))}'

    def is_binary(self):
        '''If the opcode is a binary operator'''
        return self.opcode // 100 == 2

    def is_unary(self):
        '''If the opcode is a unary operator'''
        return self.opcode // 100 == 1

    def is_valued(self):
        '''If this operation has a return value'''
        # pylint: disable=import-outside-toplevel
        from .intrinsic import PureIntrinsic, Intrinsic
        from .array import ArrayRead
        from ..array import Slice

        # Check if it's a valued intrinsic
        if isinstance(self, Intrinsic):
            # pylint: disable=import-outside-toplevel
            from .intrinsic import INTRIN_INFO
            _, _, valued, _ = INTRIN_INFO[self.opcode]
            return valued
        valued = (
            PureIntrinsic,
            FIFOPop,
            ArrayRead,
            Slice,
            Cast,
            Concat,
            Select,
            Select1Hot,
        )
        other = isinstance(self, valued)
        return other or self.is_binary() or self.is_unary()



class FIFOPop(Expr):
    '''The class for FIFO pop operation'''

    FIFO_POP = 301

    def __init__(self, fifo, meta_cond: typing.Optional[Value] = None):
        if meta_cond is None:
            # pylint: disable=import-outside-toplevel
            from .intrinsic import get_pred
            meta_cond = get_pred()
        super().__init__(FIFOPop.FIFO_POP, [fifo], meta_cond=meta_cond)

    @property
    def fifo(self):
        '''Get the FIFO port'''
        return self._operands[0]

    @property
    def dtype(self):
        '''Get the data type of the popped value'''
        return self.fifo.dtype

    def __repr__(self):
        meta = self.meta_cond
        if meta is None:
            suffix = ''
        else:
            operand = meta.as_operand() if hasattr(meta, 'as_operand') else repr(meta)
            suffix = f' // meta cond {operand}'
        return f'{self.as_operand()} = {self.fifo.as_operand()}.pop(){suffix}'

    def __getattr__(self, name):
        return self.dtype.attributize(self, name)


class Log(Expr):
    '''The class for log operation. NOTE: This operation is just like verilog $display, which is
    non-synthesizable. It is used for debugging purpose only.'''

    args: tuple  # Arguments to the log operation

    LOG = 600

    def __init__(self, fmt, *values, meta_cond=None):
        args = (fmt, *values)
        if meta_cond is None:
            # pylint: disable=import-outside-toplevel
            from .intrinsic import get_pred
            meta_cond = get_pred()
        super().__init__(Log.LOG, args, meta_cond=meta_cond)
        self.args = args

    @property
    def fmt(self):
        '''Return the format string argument.'''
        return self.args[0]

    @property
    def values(self):
        '''Return the payload values excluding the format string.'''
        return self.args[1:]

    @property
    def dtype(self):
        '''Get the data type of this operation (Void for side-effect operations)'''
        #pylint: disable=import-outside-toplevel
        from ..dtype import void
        return void()

    def __repr__(self):
        fmt = repr(self.fmt)
        payload = ", ".join(
            i.as_operand() if hasattr(i, 'as_operand') else repr(i)
            for i in self.values
        )
        base = f'log({fmt}'
        if payload:
            base += f', {payload}'
        base += ')'
        meta = self.meta_cond
        if meta is None:
            return base
        operand = meta.as_operand() if hasattr(meta, 'as_operand') else repr(meta)
        return f'{base} // meta cond {operand}'

class Concat(Expr):
    '''The class for concatenation operation, where {msb, lsb} as a right value'''

    CONCAT = 701

    def __init__(self, msb, lsb):
        super().__init__(Concat.CONCAT, [lsb, msb])

    @property
    def msb(self) -> Value:
        '''Get the most significant bit'''
        return self._operands[1]

    @property
    def lsb(self) -> Value:
        '''Get the least significant bit'''
        return self._operands[0]

    @property
    def dtype(self) -> DType:
        '''Get the data type of the concatenated value'''
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits
        return Bits(self.msb.dtype.bits + self.lsb.dtype.bits)

    def __repr__(self):
        return f'{self.as_operand()} = {{ {self.msb.as_operand()} {self.lsb.as_operand()} }}'

class Cast(Expr):
    '''The class for casting operation, including bitcast, zext, and sext.'''

    _dtype: DType  # Target data type

    BITCAST = 800
    ZEXT = 801
    SEXT = 802

    SUBCODES = {
      BITCAST: 'bitcast',
      ZEXT: 'zext',
      SEXT: 'sext',
    }

    def __init__(self, subcode, x, dtype):
        super().__init__(subcode, [x])
        self._dtype = dtype

    @property
    def dtype(self) -> DType:
        '''Get the target data type'''
        return self._dtype

    @property
    def x(self) -> Value:
        '''Get the value to cast'''
        return self._operands[0]

    def __repr__(self):
        method = Cast.SUBCODES[self.opcode]
        return f'{self.as_operand()} = {method} {self.x.as_operand()} to {self.dtype}'

@ir_builder
def log(*args):
    '''The exposed frontend function to instantiate a log operation'''
    assert isinstance(args[0], str)
    return Log(*args)


class Select(Expr):
    '''The class for the select operation'''

    # Triary operations
    SELECT = 1000

    def __init__(self, opcode, cond, true_val: Value, false_val: Value):
        assert isinstance(cond, Value), f'{type(cond)} is not a Value!'
        assert isinstance(true_val, Value), f'{type(true_val)} is not a Value!'
        assert isinstance(false_val, Value), f'{type(false_val)} is not a Value!'
        assert true_val.dtype == false_val.dtype, f'{true_val.dtype} != {false_val.dtype}'
        super().__init__(opcode, [cond, true_val, false_val])

    @property
    def cond(self) -> Value:
        '''Get the condition'''
        return self._operands[0]

    @property
    def true_value(self) -> Value:
        '''Get the true value'''
        return self._operands[1]

    @property
    def false_value(self) -> Value:
        '''Get the false value'''
        return self._operands[2]

    @property
    def dtype(self) -> DType:
        '''Get the data type of this operation'''
        return self.true_value.dtype

    def __repr__(self):
        lval = self.as_operand()
        cond = self.cond.as_operand()
        true_val = self.true_value.as_operand()
        false_val = self.false_value.as_operand()
        return f'{lval} = {cond} ? {true_val} : {false_val}'

class Select1Hot(Expr):
    '''The class for the 1hot select operation'''

    # Triary operations
    SELECT_1HOT = 1001

    def __init__(self, opcode, cond, values):
        reference = values[0]
        for i in values:
            assert reference.dtype == i.dtype, f'{reference.dtype} != {i.dtype}'
        super().__init__(opcode, [cond] + list(values))

    @property
    def dtype(self) -> DType:
        '''Get the data type of this operation'''
        return self.values[0].dtype

    @property
    def cond(self) -> Value:
        '''Get the condition'''
        return self._operands[0]

    @property
    def values(self) -> list[Value]:
        '''Get the list of possible values'''
        return self._operands[1:]

    def __repr__(self):
        lval = self.as_operand()
        cond = self.cond.as_operand()
        values = ', '.join(i.as_operand() for i in self.values)
        return f'{lval} = select_1hot {cond} ({values})'
