'''The AST node data structure for the expressions'''

#pylint: disable=cyclic-import,import-outside-toplevel

from __future__ import annotations

from functools import reduce
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

    source_name: str
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
        self.source_name = None
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
        if self.source_name is not None:
            return f'{self.source_name}'
        return f'_{namify(identifierize(self))}'

    def is_binary(self):
        '''If the opcode is a binary operator'''
        return self.opcode // 100 == 2

    def is_unary(self):
        '''If the opcode is a unary operator'''
        return self.opcode // 100 == 1

    def is_valued(self):
        '''If this operation has a return value'''
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

class BinaryOp(Expr):
    '''The class for binary operations'''

    lhs: Value  # Left-hand side operand
    rhs: Value  # Right-hand side operand

    # Binary operations
    ADD         = 200
    SUB         = 201
    MUL         = 202
    DIV         = 203
    MOD         = 204
    BITWISE_AND = 206
    BITWISE_OR  = 207
    BITWISE_XOR = 208
    ILT         = 209
    IGT         = 210
    ILE         = 211
    IGE         = 212
    EQ          = 213
    SHL         = 214
    SHR         = 215
    NEQ         = 216

    OPERATORS = {
      ADD: '+',
      SUB: '-',
      MUL: '*',
      DIV: '/',
      MOD: '%',

      ILT: '<',
      IGT: '>',
      ILE: '<=',
      IGE: '>=',
      EQ:  '==',
      NEQ: '!=',

      BITWISE_AND: '&',
      BITWISE_OR:  '|',
      BITWISE_XOR: '^',

      SHL: '<<',
      SHR: '>>',
    }

    def __init__(self, opcode, lhs, rhs):
        assert isinstance(lhs, Value), f'{type(lhs)} is not a Value!'
        assert isinstance(rhs, Value), f'{type(rhs)} is not a Value!'
        super().__init__(opcode, [lhs, rhs])

    @property
    def lhs(self):
        '''Get the left-hand side operand'''
        return self._operands[0]

    @property
    def rhs(self):
        '''Get the right-hand side operand'''
        return self._operands[1]

    @property
    def dtype(self):
        '''Get the data type of this operation'''
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits
        if self.opcode in [BinaryOp.ADD]:
            # TODO(@were): Make this bits + 1
            bits = max(self.lhs.dtype.bits, self.rhs.dtype.bits)
            tyclass = self.lhs.dtype.__class__
            return tyclass(bits)
        if self.opcode in [BinaryOp.SUB, BinaryOp.DIV, BinaryOp.MOD]:
            return type(self.lhs.dtype)(self.lhs.dtype.bits)
        if self.opcode in [BinaryOp.MUL]:
            bits = self.lhs.dtype.bits + self.rhs.dtype.bits
            tyclass = self.lhs.dtype.__class__
            return tyclass(bits)
        if self.opcode in [BinaryOp.SHL, BinaryOp.SHR]:
            return Bits(self.lhs.dtype.bits)
        if self.opcode in [BinaryOp.ILT, BinaryOp.IGT, BinaryOp.ILE, BinaryOp.IGE,
                           BinaryOp.EQ, BinaryOp.NEQ]:
            return Bits(1)
        if self.opcode in [BinaryOp.BITWISE_AND, BinaryOp.BITWISE_OR, BinaryOp.BITWISE_XOR]:
            return Bits(max(self.lhs.dtype.bits, self.rhs.dtype.bits))
        raise NotImplementedError(f'Unsupported binary operation {self.opcode}')

    def __repr__(self):
        lval = self.as_operand()
        lhs = self.lhs.as_operand()
        rhs = self.rhs.as_operand()
        op = self.OPERATORS[self.opcode]
        return f'{lval} = {lhs} {op} {rhs}'

    def is_computational(self):
        '''Check if this operation is computational'''
        return self.opcode in [BinaryOp.ADD, BinaryOp.SUB, BinaryOp.MUL, BinaryOp.DIV,
                               BinaryOp.MOD]

    def is_comparative(self):
        '''Check if this operation is comparative'''
        return self.opcode in [BinaryOp.ILT, BinaryOp.IGT, BinaryOp.ILE, BinaryOp.IGE,
                               BinaryOp.EQ, BinaryOp.NEQ]


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


class ArrayWrite(Expr):
    '''The class for array write operation, where arr[idx] = val'''

    ARRAY_WRITE = 401

    def __init__(self, arr, idx: Value, val: Value):
        super().__init__(ArrayWrite.ARRAY_WRITE, [arr, idx, val])

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

    def __repr__(self):
        return f'{self.array.as_operand()}[{self.idx.as_operand()}] = {self.val.as_operand()}'


class ArrayRead(Expr):
    '''The class for array read operation, where arr[idx] as a right value'''

    ARRAY_READ = 400

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
        from ...builder import Singleton
        from ..dtype import RecordValue

        assert isinstance(value, (Value, RecordValue)), \
            f"Value must be Value or RecordValue, got {type(value)}"

        current_module = Singleton.builder.current_module

        write_port = self.array & current_module
        return write_port._create_write(self.idx.value, value)

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

class Slice(Expr):
    '''The class for slice operation, where x[l:r] as a right value'''

    SLICE = 700

    def __init__(self, x, l: int, r: int):
        assert isinstance(l, int), f'Only int literal can slice, but got {type(l)}'
        assert isinstance(r, int), f'Only int literal can slice, but got {type(r)}'
        assert isinstance(x, Value), f'{type(x)} is not a Value!'
        from ..dtype import to_uint
        l = to_uint(l)
        r = to_uint(r)
        super().__init__(Slice.SLICE, [x, l, r])

    @property
    def x(self) -> Value:
        '''Get the value to slice'''
        return self._operands[0]

    @property
    def l(self) -> int:
        '''Get the value to slice'''
        return self._operands[1]

    @property
    def r(self) -> int:
        '''Get the value to slice'''
        return self._operands[2]

    @property
    def dtype(self) -> DType:
        '''Get the data type of the sliced value'''
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits
        from ..const import Const
        assert isinstance(self.l.value, Const)
        assert isinstance(self.r.value, Const)
        return Bits(self.r.value.value - self.l.value.value + 1)

    def __repr__(self):
        l = self.l.as_operand()
        r = self.r.as_operand()
        return f'{self.as_operand()} = {self.x.as_operand()}[{l}:{r}]'

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

class UnaryOp(Expr):
    '''The class for unary operations'''

    # Unary operations
    NEG  = 100
    FLIP = 101

    OPERATORS = {
        NEG: '-',
        FLIP: '!',
    }

    def __init__(self, opcode, x):
        super().__init__(opcode, [x])

    @property
    def x(self) -> Value:
        '''Get the operand of this unary operation'''
        return self._operands[0]

    @property
    def dtype(self) -> DType:
        '''Get the data type of this unary operation'''
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits
        return Bits(self.x.dtype.bits)

    def __repr__(self):
        return f'{self.as_operand()} = {self.OPERATORS[self.opcode]}{self.x.as_operand()}'

class PureIntrinsic(Expr):
    '''The class for accessing FIFO fields, valid, and peek'''

    # FIFO operations
    FIFO_VALID = 300
    FIFO_PEEK  = 303
    MODULE_TRIGGERED = 304
    VALUE_VALID = 305

    OPERATORS = {
        FIFO_VALID: 'valid',
        FIFO_PEEK: 'peek',
        MODULE_TRIGGERED: 'triggered',
        VALUE_VALID: 'valid',
    }

    def __init__(self, opcode, *args):
        operands = list(args)
        super().__init__(opcode, operands)

    @property
    def args(self):
        '''Get the arguments of this intrinsic'''
        return self._operands

    @property
    def dtype(self):
        '''Get the data type of this intrinsic'''
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits

        if self.opcode == PureIntrinsic.FIFO_PEEK:
            # pylint: disable=import-outside-toplevel
            from ..module import Port
            fifo = self.args[0]
            assert isinstance(fifo, Port)
            return fifo.dtype

        if self.opcode in [PureIntrinsic.FIFO_VALID, PureIntrinsic.MODULE_TRIGGERED,
                           PureIntrinsic.VALUE_VALID]:
            return Bits(1)

        raise NotImplementedError(f'Unsupported intrinsic operation {self.opcode}')

    def __repr__(self):
        if self.opcode in [PureIntrinsic.FIFO_PEEK, PureIntrinsic.FIFO_VALID,
                           PureIntrinsic.MODULE_TRIGGERED, PureIntrinsic.VALUE_VALID]:
            fifo = self.args[0].as_operand()
            return f'{self.as_operand()} = {fifo}.{self.OPERATORS[self.opcode]}()'
        raise NotImplementedError

    def __getattr__(self, name):
        if self.opcode == PureIntrinsic.FIFO_PEEK:
            port = self.args[0]
            # pylint: disable=import-outside-toplevel
            from ..module import Port
            assert isinstance(port, Port)
            return port.dtype.attributize(self, name)

        assert False, f"Cannot access attribute {name} on {self}"
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

def concat(*args: typing.List[Value]):
    """
    Concatenate multiple arguments using the existing concat method.
    This function translates concat(a, b, c) into a.concat(b).concat(c).
    """
    if len(args) < 2:
        raise ValueError("concat requires at least two arguments")
    return reduce(lambda x, y: x.concat(y), args)
