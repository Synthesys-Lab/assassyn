from .builder import ir_builder

class Expr(object):

    def __init__(self, opcode):
        self.opcode = opcode
        self._id = None

    @property
    def id(self):
        return self._id

    @id.setter
    def id(self, value):
        self._id = value

    def as_operand(self):
        return f'_{self.id}'

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
        return BinaryOp(BinaryOp.BIT_OR, self, other)

    @ir_builder(node_type='expr')
    def __rxor__(self, other):
        return BinaryOp(BinaryOp.BIT_XOR, self, other)

    @ir_builder(node_type='expr')
    def __rand__(self, other):
        return BinaryOp(BinaryOp.BIT_AND, self, other)

class BinaryOp(Expr):

    # Binary operations
    ADD = 20
    SUB = 21
    MUL = 22
    DIV = 23
    MOD = 24
    BIT_AND = 26
    BIT_OR = 27
    BIT_XOR = 28
    # Array operations
    ARRAY_READ = 40

    OPERATORS = {
      ADD: '+',
      SUB: '-',
      MUL: '*',
      DIV: '/',
      MOD: '%',
      BIT_AND: '&',
      BIT_OR: '|',
      BIT_XOR: '^',
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
    FIFO_PUSH = 32
    ARRAY_WRITE = 41
    LOG = 60

    def __init__(self, opcode, *args):
        super().__init__(opcode)
        self.args = args

    def __repr__(self):
        if self.opcode == self.LOG:
            fmt = repr(self.args[0])
            return f'log({fmt}, {", ".join(self.args[1:].as_operand())}'
        elif self.opcode == self.ARRAY_WRITE:
            arr = self.args[0].as_operand()
            idx = self.args[1].as_operand()
            val = self.args[2].as_operand()
            return f'{arr}[{idx}] = {val}'
        # FIFO_PUSH
        fifo = self.args[0].as_operand()
        val = self.args[1].as_operand()
        return f'{fifo}.push({val})'

def log(*args):
    assert isinstance(args[0], str)
    return SideEffect(SideEffect.LOG, *args)

class UnaryOp(Expr):
    # Unary operations
    NEG = 10
    FLIP = 11
    # Call operations
    ASYNC_CALL = 50
    # FIFO operations
    FIFO_VALID = 30
    FIFO_POP = 31
    FIFO_PEEK = 33

    def __init__(self, opcode, x):
        super().__init__(opcode)
        self.x = x


class BindInst(Expr):

    BIND = 51

    def bind(self, **kwargs):
        self.args.update(kwargs)

    def __init__(self, callee, **kwargs):
        super().__init__(0)
        self.callee = callee
        self.args = dict(kwargs)
