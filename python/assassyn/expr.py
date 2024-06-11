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
    # Array operations
    ARRAY_READ = 400

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
        if self.opcode == self.ARRAY_READ:
            return f'{lval} = {self.lhs.as_operand()}[{self.rhs.as_operand()}]'
        lhs = self.lhs.as_operand()
        rhs = self.rhs.as_operand()
        return f'{lval} = {lhs} {self.OPERATORS[self.opcode]} {rhs}'

class SideEffect(Expr):

    # Side effects
    FIFO_PUSH  = 302
    FIFO_POP   = 301
    ARRAY_WRITE = 401
    LOG = 600

    def __init__(self, opcode, *args):
        super().__init__(opcode)
        self.args = args

    def __repr__(self):
        if self.opcode == self.LOG:
            fmt = repr(self.args[0])
            return f'log({fmt}, {", ".join(i.as_operand() for i in self.args[1:])})'
        elif self.opcode == self.ARRAY_WRITE:
            arr = self.args[0].as_operand()
            idx = self.args[1].as_operand()
            val = self.args[2].as_operand()
            return f'{arr}[{idx}] = {val}'
        # FIFO_PUSH
        elif self.opcode == self.FIFO_PUSH:
            fifo = self.args[0].as_operand()
            val = self.args[1].as_operand()
            return f'{fifo}.push({val})'
        # FIFO_POP
        assert self.opcode == self.FIFO_POP
        fifo = self.args[0].as_operand()
        return f'{fifo}.pop()'

@ir_builder(node_type='expr')
def log(*args):
    assert isinstance(args[0], str)
    return SideEffect(SideEffect.LOG, *args)

class UnaryOp(Expr):
    # Unary operations
    NEG  = 100
    FLIP = 101
    # Call operations
    ASYNC_CALL = 500

    def __init__(self, opcode, x):
        super().__init__(opcode)
        self.x = x

class FIFOField(Expr):
    # FIFO operations
    FIFO_VALID = 300
    FIFO_PEEK  = 303

    def __init__(self, opcode, fifo):
        super().__init__(opcode)
        self.fifo = fifo

class BindInst(Expr):

    BIND = 51

    def bind(self, **kwargs):
        self.args.update(kwargs)

    def __init__(self, callee, **kwargs):
        super().__init__(0)
        self.callee = callee
        self.args = dict(kwargs)

def is_fifo_related(opcode):
    return opcode // 100 == 3

def is_binary(opcode):
    return opcode // 100 == 2

def is_unary(opcode):
    return opcode // 100 == 1

def is_valued(opcode):
    other = [FIFOField.FIFO_PEEK, FIFOField.FIFO_VALID, BinaryOp.ARRAY_READ, SideEffect.FIFO_POP]
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
    BinaryOp.ARRAY_READ: 'array_read',

    UnaryOp.FLIP: 'flip',
    UnaryOp.NEG: 'neg',

    FIFOField.FIFO_PEEK: 'peek',
    FIFOField.FIFO_VALID: 'valid',

    SideEffect.FIFO_POP: 'pop',
    SideEffect.FIFO_PUSH: 'push',
    SideEffect.ARRAY_WRITE: 'array_write',
    SideEffect.LOG: 'log',
}

def opcode_to_ib(opcode):
    if is_fifo_related(opcode):
        return f'create_fifo_{CG_OPCODE[opcode]}'
    return f'create_{CG_OPCODE[opcode]}'

