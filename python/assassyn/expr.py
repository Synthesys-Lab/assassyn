from .builder import ir_builder, Singleton
from .value import Value

class Expr(Value):

    def __init__(self, opcode):
        self.opcode = opcode

    def as_operand(self):
        return f'_{hex(id(self))[-5:-1]}'

    def is_fifo_related(self):
        return self.opcode // 100 == 3

    def is_binary(self):
        return self.opcode // 100 == 2

    def is_unary(self):
        return self.opcode // 100 == 1

    def is_valued(self):
        other = isinstance(self, (FIFOField, FIFOPop, ArrayRead, Slice, Cast))
        return other or self.is_binary() or self.is_unary()

class BinaryOp(Expr):

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

      BITWISE_AND: '&',
      BITWISE_OR:  '|',
      BITWISE_XOR: '^',
    }

    def __init__(self, opcode, lhs, rhs):
        super().__init__(opcode)
        self.lhs = lhs
        self.rhs = rhs

    def __repr__(self):
        lval = self.as_operand()
        lhs = self.lhs.as_operand()
        rhs = self.rhs.as_operand()
        return f'{lval} = {lhs} {self.OPERATORS[self.opcode]} {rhs}'

class FIFOPush(Expr):

    FIFO_PUSH  = 302

    def __init__(self, fifo, val):
        super().__init__(FIFOPush.FIFO_PUSH)
        self.fifo = fifo
        self.val = val
        self.bind = None

    def __repr__(self):
        handle = self.as_operand()
        return f'{self.fifo.as_operand()}.push({self.val.as_operand()}) // handle = {handle}'

class FIFOPop(Expr):

    FIFO_POP = 301

    def __init__(self, fifo):
        super().__init__(FIFOPop.FIFO_POP)
        self.fifo = fifo

    def __repr__(self):
        return f'{self.as_operand()} = {self.fifo.as_operand()}.pop()'


class ArrayWrite(Expr):

    ARRAY_WRITE = 401

    def __init__(self, arr, idx, val):
        super().__init__(ArrayWrite.ARRAY_WRITE)
        self.arr = arr
        self.idx = idx
        self.val = val

    def __repr__(self):
        return f'{self.arr.as_operand()}[{self.idx.as_operand()}] = {self.val.as_operand()}'


class ArrayRead(Expr):

    ARRAY_READ = 400

    def __init__(self, arr, idx):
        super().__init__(ArrayRead.ARRAY_READ)
        self.arr = arr
        self.idx = idx

    def __repr__(self):
        return f'{self.as_operand()} = {self.arr.as_operand()}[{self.idx.as_operand()}]'

class Log(Expr):

    LOG = 600

    def __init__(self, *args):
        super().__init__(Log.LOG)
        self.args = args

    def __repr__(self):
        fmt = repr(self.args[0])
        return f'log({fmt}, {", ".join(i.as_operand() for i in self.args[1:])})'

class Slice(Expr):

    SLICE = 700

    def __init__(self, x, l: int, r: int):
        assert isinstance(l, int) and isinstance(r, int) and l <= r
        super().__init__(Slice.SLICE)
        self.x = x
        from .dtype import to_uint
        self.l = to_uint(l)
        self.r = to_uint(r)

    def __repr__(self):
        return f'{self.as_operand()} = {self.x.as_operand()}[{self.l}:{self.r}]'

class Concat(Expr):

    CONCAT = 701

    def __init__(self, msb, lsb):
        super().__init__(Concat.CONCAT)
        self.msb = msb
        self.lsb = lsb

    def __repr__(self):
        return f'{self.as_operand()} = {self.msb.as_operand()} |.| {self.lsb.as_operand()}'

class Cast(Expr):

    BITCAST = 800

    def __init__(self, subcode, x, dtype):
        super().__init__(subcode)
        self.x = x
        self.dtype = dtype

@ir_builder(node_type='expr')
def log(*args):
    assert isinstance(args[0], str)
    return Log(*args)

class UnaryOp(Expr):
    # Unary operations
    NEG  = 100
    FLIP = 101

    OPERATORS = {
        NEG: '-',
        FLIP: '~',
    }

    def __init__(self, opcode, x):
        super().__init__(opcode)
        self.x = x

    def __repr__(self):
        return f'{self.as_operand()} = {self.OPERATORS[self.opcode]}{self.x.as_operand()}'

class FIFOField(Expr):
    # FIFO operations
    FIFO_VALID = 300
    FIFO_PEEK  = 303

    def __init__(self, opcode, fifo):
        super().__init__(opcode)
        self.fifo = fifo


class Bind(Expr):
    BIND = 501

    def _push(self, **kwargs):
        for k, v in kwargs.items():
            push = getattr(self.callee, k).push(v)
            push.bind = self
            self.pushes.append(push)

    @ir_builder(node_type='expr')
    def bind(self, **kwargs):
        self._push(**kwargs)
        return self

    @ir_builder(node_type='expr')
    def async_called(self):
        return AsyncCall(self)

    def __init__(self, callee, **kwargs):
        super().__init__(Bind.BIND)
        self.callee = callee
        self.pushes = []
        self._push(**kwargs)

    def __repr__(self):
        args = ', '.join(f'{v.as_operand()} /* {v.fifo.as_operand()}={v.val.as_operand()} */' for v in self.pushes)
        callee = self.callee.as_operand()
        lval = self.as_operand()
        return f'{lval} = {callee}.bind[ {args} ]'

class AsyncCall(Expr):
    # Call operations
    ASYNC_CALL = 500

    def __init__(self, bind: Bind):
        super().__init__(AsyncCall.ASYNC_CALL)
        self.bind = bind

    def __repr__(self):
        bind = self.bind.as_operand()
        return f'async_call {bind}'

