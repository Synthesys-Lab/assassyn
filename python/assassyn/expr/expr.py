'''The AST node data structure for the expressions'''

#pylint: disable=cyclic-import,import-outside-toplevel

from functools import reduce

from ..builder import ir_builder
from ..value import Value
from ..utils import identifierize

class Expr(Value):
    '''The frontend base node for expressions'''

    def __init__(self, opcode):
        '''Initialize the expression with an opcode'''
        self.opcode = opcode

    def as_operand(self):
        '''Dump the expression as an operand'''
        return f'_{identifierize(self)}'

    def is_binary(self):
        '''If the opcode is a binary operator'''
        return self.opcode // 100 == 2

    def is_unary(self):
        '''If the opcode is a unary operator'''
        return self.opcode // 100 == 1

    def is_valued(self):
        '''If this operation has a return value'''
        valued = (PureInstrinsic, FIFOPop, ArrayRead, Slice, Cast, Concat, Select, Select1Hot)
        other = isinstance(self, valued)
        return other or self.is_binary() or self.is_unary()

class BinaryOp(Expr):
    '''The class for binary operations'''

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
        super().__init__(opcode)
        self.lhs = lhs
        self.rhs = rhs

    def __repr__(self):
        lval = self.as_operand()
        lhs = self.lhs.as_operand()
        rhs = self.rhs.as_operand()
        return f'{lval} = {lhs} {self.OPERATORS[self.opcode]} {rhs}'

class FIFOPush(Expr):
    '''The class for FIFO push operation'''

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
    '''The class for FIFO pop operation'''

    FIFO_POP = 301

    def __init__(self, fifo):
        super().__init__(FIFOPop.FIFO_POP)
        self.fifo = fifo

    def __repr__(self):
        return f'{self.as_operand()} = {self.fifo.as_operand()}.pop()'


class ArrayWrite(Expr):
    '''The class for array write operation, where arr[idx] = val'''

    ARRAY_WRITE = 401

    def __init__(self, arr, idx, val):
        super().__init__(ArrayWrite.ARRAY_WRITE)
        self.arr = arr
        self.idx = idx
        self.val = val

    def __repr__(self):
        return f'{self.arr.as_operand()}[{self.idx.as_operand()}] = {self.val.as_operand()}'


class ArrayRead(Expr):
    '''The class for array read operation, where arr[idx] as a right value'''

    ARRAY_READ = 400

    def __init__(self, arr, idx):
        super().__init__(ArrayRead.ARRAY_READ)
        self.arr = arr
        self.idx = idx

    def __repr__(self):
        return f'{self.as_operand()} = {self.arr.as_operand()}[{self.idx.as_operand()}]'

class Log(Expr):
    '''The class for log operation. NOTE: This operation is just like verilog $display, which is
    non-synthesizable. It is used for debugging purpose only.'''

    LOG = 600

    def __init__(self, *args):
        super().__init__(Log.LOG)
        self.args = args

    def __repr__(self):
        fmt = repr(self.args[0])
        return f'log({fmt}, {", ".join(i.as_operand() for i in self.args[1:])})'

class Slice(Expr):
    '''The class for slice operation, where x[l:r] as a right value'''

    SLICE = 700

    def __init__(self, x, l: int, r: int):
        assert isinstance(l, int) and isinstance(r, int) and l <= r
        super().__init__(Slice.SLICE)
        self.x = x
        from ..dtype import to_uint
        self.l = to_uint(l)
        self.r = to_uint(r)

    def __repr__(self):
        return f'{self.as_operand()} = {self.x.as_operand()}[{self.l}:{self.r}]'

class Concat(Expr):
    '''The class for concatenation operation, where {msb, lsb} as a right value'''

    CONCAT = 701

    def __init__(self, msb, lsb):
        super().__init__(Concat.CONCAT)
        self.msb = msb
        self.lsb = lsb

    def __repr__(self):
        return f'{self.as_operand()} = {{ {self.msb.as_operand()} {self.lsb.as_operand()} }}'

class Cast(Expr):
    '''The class for casting operation, including bitcast, zext, and sext.'''

    BITCAST = 800
    ZEXT = 801
    SEXT = 802

    SUBCODES = {
      BITCAST: 'bitcast',
      ZEXT: 'zext',
      SEXT: 'sext',
    }

    def __init__(self, subcode, x, dtype):
        super().__init__(subcode)
        self.x = x
        self.dtype = dtype

    def __repr__(self):
        method = Cast.SUBCODES[self.opcode]
        return f'{self.as_operand()} = {method} {self.x.as_operand()} to {self.dtype}'

@ir_builder(node_type='expr')
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
        FLIP: '~',
    }

    def __init__(self, opcode, x):
        super().__init__(opcode)
        self.x = x

    def __repr__(self):
        return f'{self.as_operand()} = {self.OPERATORS[self.opcode]}{self.x.as_operand()}'

class PureInstrinsic(Expr):
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
        super().__init__(opcode)
        self.args = list(args)

    def __repr__(self):
        if self.opcode in [PureInstrinsic.FIFO_PEEK, PureInstrinsic.FIFO_VALID,
                           PureInstrinsic.MODULE_TRIGGERED, PureInstrinsic.VALUE_VALID]:
            fifo = self.args[0].as_operand()
            return f'{self.as_operand()} = {fifo}.{self.OPERATORS[self.opcode]}()'
        raise NotImplementedError


class Bind(Expr):
    '''The class for binding operations. Function bind is a functional programming concept like
    Python's `functools.partial`.'''

    BIND = 501

    def _push(self, **kwargs):
        for k, v in kwargs.items():
            push = getattr(self.callee, k).push(v)
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

    @ir_builder(node_type='expr')
    def async_called(self, **kwargs):
        '''The exposed frontend function to instantiate an async call operation'''
        self._push(**kwargs)
        return AsyncCall(self)

    def __init__(self, callee, **kwargs):
        super().__init__(Bind.BIND)
        self.callee = callee
        self.pushes = []
        self._push(**kwargs)
        self.fifo_depths = {}

    def set_fifo_depth(self, **kwargs):
        """Set FIFO depths using keyword arguments."""
        for name, depth in kwargs.items():
            if not isinstance(depth, int):
                raise ValueError(f"Depth for {name} must be an integer")
            for push in self.pushes:
                if push.fifo.name == name:
                    self.fifo_depths[self.as_operand()] = depth
                    push.fifo_depth = depth
                    break
            else:
                raise ValueError(f"No push found for FIFO named {name}")
        return self

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
        super().__init__(AsyncCall.ASYNC_CALL)
        self.bind = bind

    def __repr__(self):
        bind = self.bind.as_operand()
        return f'async_call {bind}'

class Select(Expr):
    '''The class for the select operation'''

    # Triary operations
    SELECT = 1000

    def __init__(self, opcode, cond, true_val, false_val):
        super().__init__(opcode)
        self.cond = cond
        self.true_value = true_val
        self.false_value = false_val

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
        super().__init__(opcode)
        self.cond = cond
        self.values = list(values)

    def __repr__(self):
        lval = self.as_operand()
        cond = self.cond.as_operand()
        values = ', '.join(i.as_operand() for i in self.values)
        return f'{lval} = select_1hot {cond} ({values})'

def concat(*args):
    """
    Concatenate multiple arguments using the existing concat method.
    This function translates concat(a, b, c) into a.concat(b).concat(c).
    """
    if len(args) < 2:
        raise ValueError("concat requires at least two arguments")
    return reduce(lambda x, y: x.concat(y), args)
