from decorator import decorator

from .builder import Singleton, ir_builder
from .dtype import DType
from .block import Block
from .expr import Expr, BindInst

@decorator
def combinational(func, *args, **kwargs):
    builder = Singleton.builder
    args[0].body = Block()
    builder.insert_point['expr'] = args[0].body.body
    return func(*args, **kwargs)

@decorator
def wait_until(func, *args, **kwargs):
    pass

@decorator
def constructor(func, *args, **kwargs):
    builder = Singleton.builder
    builder.modules.append(func(*args, **kwargs))
    # Set the name of the ports.
    for k, v in args[0].__dict__.items():
        if isinstance(v, Port):
            v.name = k

class Module(object):
    def __init__(self):
        self.name = type(self).__name__
        pass

    @ir_builder(node_type='expr')
    def async_called(self, *args, **kwargs):
        return Expr(Expr.ASYNC_CALL, self, *args, **args)

    @ir_builder(node_type='expr')
    def bind(self, **kwargs):
        return BindInst(self, **kwargs)

class Port(object):
    def __init__(self, dtype: DType):
        assert isinstance(dtype, DType)
        self.dtype = dtype

    @ir_builder(node_type='expr')
    def valid(self):
        return Expr(Expr.FIFO_VALID, self)

    @ir_builder(node_type='expr')
    def peek(self):
        return Expr(Expr.FIFO_PEEK, self)

    @ir_builder(node_type='expr')
    def pop(self):
        return Expr(Expr.FIFO_POP, self)

    @ir_builder(node_type='expr')
    def push(self):
        return Expr(Expr.FIFO_PUSH, self)

