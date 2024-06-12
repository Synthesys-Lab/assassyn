from .builder import ir_builder, Singleton


class Expr(object):

    def __init__(self, opcode):
        self.opcode = opcode

    def as_operand(self):
        return f'_{hex(id(self))[-5:-1]}'

    @ir_builder(node_type='expr')
    def __add__(self, other):
        return BinaryOp(BinaryOp.ADD, self, other)

    @ir_builder(node_type='expr')
    def __sub__(self, other):
        return BinaryOp(BinaryOp.SUB, self, other)

    @ir_builder(node_type='expr')
    def __mul__(self, other):
        return BinaryOp(BinaryOp.MUL, self, other)

    @ir_builder(node_type='expr')
    def __ror__(self, other):
        return BinaryOp(BinaryOp.BITWISE_OR, self, other)

    @ir_builder(node_type='expr')
    def __rxor__(self, other):
        return BinaryOp(BinaryOp.BITWISE_XOR, self, other)

    @ir_builder(node_type='expr')
    def __rand__(self, other):
        return BinaryOp(BinaryOp.BITWISE_AND, self, other)

class BinaryOp(Expr):

    # Binary operations
    ADD     = 200
    SUB     = 201
    MUL     = 202
    DIV     = 203
    MOD     = 204
    BITWISE_AND = 206
    BITWISE_OR  = 207
    BITWISE_XOR = 208

    OPERATORS = {
      ADD: '+',
      SUB: '-',
      MUL: '*',
      DIV: '/',
      MOD: '%',
      BITWISE_AND: '&',
      BITWISE_OR: '|',
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

    def __repr__(self):
        return f'{self.fifo.as_operand()}.push({self.val.as_operand()})'

class FIFOPop(Expr):

    FIFO_POP = 301

    def __init__(self, fifo):
        super().__init__(FIFOPop.FIFOPOP)
        self.fifo = fifo

    def __repr__(self):
        return f'{self.fifo.as_operand()}.pop()'


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

@ir_builder(node_type='expr')
def log(*args):
    assert isinstance(args[0], str)
    return Log(*args)

class UnaryOp(Expr):
    # Unary operations
    NEG  = 100
    FLIP = 101
    # Call operations
    ASYNC_CALL = 500

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

class BindInst(Expr):

    BIND = 501

    def bind(self, *args, **kwargs):
        if not len(args):
            self.args.update(kwargs)
        elif not len(kwargs):
            self.args.update(args)

    def __init__(self, callee, *args, **kwargs):
        super().__init__(BindInst.BIND)
        self.callee = callee
        self.args = dict(kwargs)

def is_fifo_related(opcode):
    return opcode // 100 == 3

def is_binary(opcode):
    return opcode // 100 == 2

def is_unary(opcode):
    return opcode // 100 == 1

def is_valued(opcode):
    other = [FIFOField.FIFO_PEEK, FIFOField.FIFO_VALID, ArrayRead.ARRAY_READ, FIFOPop.FIFO_POP]
    return is_binary(opcode) or is_binary(opcode) or opcode in other

CG_OPCODE = {
    BinaryOp.ADD: 'add',
    BinaryOp.SUB: 'sub',
    BinaryOp.MUL: 'mul',
    BinaryOp.DIV: 'div',
    BinaryOp.MOD: 'mod',
    BinaryOp.BITWISE_AND: 'bitwise_and',
    BinaryOp.BITWISE_OR: 'bitwise_or',
    BinaryOp.BITWISE_XOR: 'bitwise_xor',

    UnaryOp.FLIP: 'flip',
    UnaryOp.NEG: 'neg',

    FIFOField.FIFO_PEEK: 'peek',
    FIFOField.FIFO_VALID: 'valid',

    FIFOPop.FIFO_POP: 'pop',
    FIFOPush.FIFO_PUSH: 'push',

    ArrayRead.ARRAY_READ: 'array_read',
    ArrayWrite.ARRAY_WRITE: 'array_write',

    Log.LOG: 'log',
}

def opcode_to_ib(opcode):
    if is_fifo_related(opcode):
        return f'create_fifo_{CG_OPCODE[opcode]}'
    return f'create_{CG_OPCODE[opcode]}'

