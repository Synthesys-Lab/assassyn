from decorator import decorator
import inspect

from .builder import Singleton, ir_builder
from .dtype import DType
from .block import Block
from .expr import Expr, Bind, FIFOPop, FIFOField, FIFOPush, AsyncCall


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
            v.module = args[0]

class Module(object):
    IMPLICIT_POP = 0
    EXPLICIT_POP = 1

    def __init__(self):
        self.name = type(self).__name__
        if self.name not in ['Driver', 'Testbench']:
            self.name = self.name + hex(id(self))[-5:-1]
        self.body = None
        self.linearize_ptr = {}

    @property
    def ports(self):
        return [v for _, v in self.__dict__.items() if isinstance(v, Port)]

    @ir_builder(node_type='expr')
    def async_called(self, **kwargs):
        bind = self.bind(**kwargs)
        return AsyncCall(bind)

    @ir_builder(node_type='expr')
    def bind(self, **kwargs):
        return Bind(self, **kwargs)

    def as_operand(self):
        return self.name

    def __repr__(self):
        Singleton.linearize_ptr = self.linearize_ptr
        ports = '\n    '.join(repr(v) for v in self.ports)
        if ports:
            ports = f'{{\n    {ports}\n  }} '
        Singleton.repr_ident = 2
        body = self.body.__repr__()
        return f'  module {self.name} {ports}{{\n{body}\n  }}'

class Port(object):
    def __init__(self, dtype: DType):
        assert isinstance(dtype, DType)
        self.dtype = dtype
        self.name = self.module = None

    @ir_builder(node_type='expr')
    def valid(self):
        return FIFOField(Expr.FIFO_VALID, self)

    @ir_builder(node_type='expr')
    def peek(self):
        return FIFOField(Expr.FIFO_PEEK, self)

    @ir_builder(node_type='expr')
    def pop(self):
        return FIFOPop(self)

    @ir_builder(node_type='expr')
    def push(self, v):
        return FIFOPush(self, v)

    def __repr__(self):
        return f'{self.name}: port<{self.dtype}>'

    def as_operand(self):
        return f'{self.module.name}.{self.name}'

@decorator
def combinational(func, port=Module.IMPLICIT_POP, *args, **kwargs):
    args[0].body = Block(Block.MODULE_ROOT)
    Singleton.builder.insert_point['expr'] = args[0].body.body
    Singleton.builder.cur_module = args[0]
    Singleton.builder.builder_func = func
    if port == Module.IMPLICIT_POP:
        restore = {}
        for k, v in args[0].__dict__.items():
            if isinstance(v, Port):
                restore[k] = v
                setattr(args[0], k, v.pop())
    res = func(*args, **kwargs)
    if port == Module.IMPLICIT_POP:
        for k, v in restore.items():
            setattr(args[0], k, v)
    Singleton.builder.cleanup_symtab()
    return res

