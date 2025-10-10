'''The AST node data structure for the expressions'''

#pylint: disable=cyclic-import,import-outside-toplevel

from __future__ import annotations

import typing

from ...builder import ir_builder
from ..value import Value
from ...utils import namify, identifierize

if typing.TYPE_CHECKING:
    from ..array import Array
    from ..module import Port, Module, Wire
    from ..dtype import DType
    from ..block import Block, CondBlock

class Operand:
    '''The base class for all operands. It is used to dump the operand as a string.'''
    _value: Value # The value of this operand
    _user: typing.Union[Expr, CondBlock] # The user of this operand

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
    parent: typing.Optional[Block]  # Parent block of this expression
    users: typing.List[Operand]  # List of users of this expression
    _operands: typing.List[
        typing.Union[Operand, Port, Array, int]
    ] # List of operands of this expression

    def __init__(self, opcode, operands: list):
        '''Initialize the expression with an opcode'''
        #pylint: disable=import-outside-toplevel,too-many-locals
        from ..array import Array
        from ..const import Const
        from ..module import Port, Wire, Module
        from ..dtype import RecordValue
        from ...builder import Singleton
        from ..module.downstream import Downstream
        from .call import Bind
        self.opcode = opcode
        self.loc = self.parent = None
        # NOTE: We only wrap values in Operand, not Ports or Arrays
        self._operands = []
        for i in operands:
            wrapped = i
            if isinstance(i, (Array, Port, Wire)):
                i.users.append(self)
            elif isinstance(i, Expr):
                if isinstance(i, Bind):
                    wrapped = Operand(i, self)
                    i.users.append(wrapped)
                else:
                    current_module = Singleton.builder.current_module
                    expr_module = i.parent.module if i.parent else None
                    if not isinstance(current_module, Downstream):
                        assert current_module == expr_module, \
                            f'Expression {i} is from module {expr_module}, \
                            but current module is {current_module}'
                    wrapped = Operand(i, self)
                    i.users.append(wrapped)
            elif isinstance(i, (Const, str, RecordValue)):
                wrapped = Operand(i, self)
            elif isinstance(i, Module):
                wrapped = Operand(i, self)
            elif isinstance(i, Downstream):
                wrapped = Operand(i, self)
            else:
                assert False, f'{i} is a {type(i)}'
            self._operands.append(wrapped)
        self.users = []

    def get_operand(self, idx: int):
        '''Get the operand at the given index'''
        if idx < 0 or idx >= len(self._operands):
            raise IndexError(f'Index {idx} out of range for {self}')
        return self._operands[idx]

    @property
    def operands(self):
        '''Get the operands of this expression'''
        return self._operands

    def as_operand(self):
        '''Dump the expression as an operand'''
        # Check if a meaningful name has been assigned by the naming system
        # Use __dict__ directly to avoid triggering __getattr__
        semantic_name_attr = '__assassyn_semantic_name__'
        if semantic_name_attr in self.__dict__ and self.__dict__[semantic_name_attr] is not None:
            return self.__dict__[semantic_name_attr]
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
            WireRead,
        )
        other = isinstance(self, valued)
        return other or self.is_binary() or self.is_unary()



class FIFOPop(Expr):
    '''The class for FIFO pop operation'''

    FIFO_POP = 301

    def __init__(self, fifo):
        super().__init__(FIFOPop.FIFO_POP, [fifo])

    @property
    def fifo(self):
        '''Get the FIFO port'''
        return self._operands[0]

    @property
    def dtype(self):
        '''Get the data type of the popped value'''
        return self.fifo.dtype

    def __repr__(self):
        return f'{self.as_operand()} = {self.fifo.as_operand()}.pop()'

    def __getattr__(self, name):
        return self.dtype.attributize(self, name)


class Log(Expr):
    '''The class for log operation. NOTE: This operation is just like verilog $display, which is
    non-synthesizable. It is used for debugging purpose only.'''

    args: tuple  # Arguments to the log operation

    LOG = 600

    def __init__(self, *args):
        super().__init__(Log.LOG, args)
        self.args = args

    def __repr__(self):
        fmt = repr(self.args[0])
        return f'log({fmt}, {", ".join(i.as_operand() for i in self.args[1:])})'

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

    dtype: DType  # Target data type

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
        self.dtype = dtype

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

class WireAssign(Expr):
    '''The class for wire assignment operations'''

    WIRE_ASSIGN = 1100

    def __init__(self, wire, value):
        super().__init__(WireAssign.WIRE_ASSIGN, [wire, value])

    @property
    def wire(self):
        '''Get the wire being assigned to'''
        return self._operands[0]

    @property
    def value(self):
        '''Get the value being assigned'''
        return self._operands[1]

    def __repr__(self):
        return f'{self.wire.as_operand()} = {self.value.as_operand()}'

@ir_builder
def wire_assign(wire, value):
    '''Create a wire assignment expression'''
    return WireAssign(wire, value)


class WireRead(Expr):
    '''The class for reading from an external wire.'''

    WIRE_READ = 1101

    def __init__(self, wire):
        super().__init__(WireRead.WIRE_READ, [wire])

    @property
    def wire(self):
        '''Return the wire being read.'''
        return self._operands[0]

    @property
    def dtype(self):
        '''The data type carried by the wire.'''
        return getattr(self.wire, 'dtype', None)

    def __repr__(self):
        return f'{self.as_operand()} = {self.wire.as_operand()}'


@ir_builder
def wire_read(wire):
    '''Create a wire read expression.'''
    return WireRead(wire)
