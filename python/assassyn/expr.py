from .builder import ir_builder

class Expr(object):

    FIFO_PUSH = 32
    # Array operations
    ARRAY_READ = 40
    ARRAY_WRITE = 41

    def __init__(self, opcode):
        self.opcode = opcode

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
        return BinaryOp(Expr.BIT_XOR, self, other)

    @ir_builder(node_type='expr')
    def __rand__(self, other):
        return BinaryOp(Expr.BIT_AND, self, other)

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

    def __init__(self, opcode, lhs, rhs):
        super().__init__(opcode)
        self.lhs = lhs
        self.rhs = rhs

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
