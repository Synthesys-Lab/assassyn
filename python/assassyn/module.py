from decorator import decorator
import inspect

from .builder import Singleton, ir_builder
from .dtype import DType
from .block import Block
from .expr import Expr, BindInst, SideEffect, FIFOField


@decorator
def wait_until(func, *args, **kwargs):
    pass

@decorator
def constructor(func, *args, **kwargs):
    builder = Singleton.builder
    super(type(args[0]), args[0]).__init__()
    func(*args, **kwargs)
    builder.insert_point['module'].append(args[0])
    builder.insert_point['expr'] = args[0].body
    # Set the name of the ports.
    for k, v in args[0].__dict__.items():
        if isinstance(v, Port):
            v.name = k

class Module(object):
    IMPLICIT_POP = 0
    EXPLICIT_POP = 1

    def __init__(self):
        self.name = type(self).__name__
        self.body = None
        self.linearize_ptr = {}

    @ir_builder(node_type='expr')
    def async_called(self, *args, **kwargs):
        return Expr(Expr.ASYNC_CALL, self, *args, **args)

    @ir_builder(node_type='expr')
    def bind(self, **kwargs):
        return BindInst(self, **kwargs)

    def __repr__(self):
        Singleton.linearize_ptr = self.linearize_ptr
        body = '    ' + '\n    '.join(repr(elem) for elem in self.body.body)
        return f'  module {self.name} {{\n{body}\n  }}'

class Port(object):
    def __init__(self, dtype: DType):
        assert isinstance(dtype, DType)
        self.dtype = dtype

    @ir_builder(node_type='expr')
    def valid(self):
        return FIFOField(Expr.FIFO_VALID, self)

    @ir_builder(node_type='expr')
    def peek(self):
        return FIFOField(Expr.FIFO_PEEK, self)

    @ir_builder(node_type='expr')
    def pop(self):
        return SideEffect(Expr.FIFO_POP, self)

    @ir_builder(node_type='expr')
    def push(self):
        return SideEffect(Expr.FIFO_PUSH, self)

@decorator
def combinational(func, port=Module.IMPLICIT_POP, *args, **kwargs):
    args[0].body = Block(Block.MODULE_ROOT)
    Singleton.builder.insert_point['expr'] = args[0].body.body
    Singleton.builder.cur_module = args[0]
    Singleton.builder.builder_func = func
    res = func(*args, **kwargs)
    Singleton.builder.cleanup_symtab()
    return res

