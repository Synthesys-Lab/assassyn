'''The module for defining the AST nodes for the module and ports.'''

from decorator import decorator

from .builder import Singleton, ir_builder
from .dtype import DType
from .block import Block
from .expr import Bind, FIFOPop, FIFOField, FIFOPush, AsyncCall
from .expr.intrinsic import _wait_until

@decorator
def constructor(func, *args, **kwargs):
    '''A decorator for marking a function as a constructor of a module.'''
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

@decorator
def wait_until(func, *args, **kwargs):
    '''A decorator for marking a module with wait-until logic.'''
    #pylint: disable=protected-access
    module_self = args[0]
    assert isinstance(module_self, Module)
    assert Singleton.builder.cur_module is module_self
    # Prepare for wait-until logic.
    pops = []
    for elem in module_self.body.body:
        assert isinstance(elem, FIFOPop)
        pops.append(elem)
    module_self._flip_restore()
    module_self.body.body.clear()
    cond = func(*args, **kwargs)
    res = _wait_until(cond)
    module_self._flip_restore()
    # Restore the FIFO.pop operations.
    for elem in pops:
        module_self.body.body.append(elem)
    Singleton.builder.cur_module.attrs.add(Module.ATTR_WAIT_UNTIL)
    return res


class Module:
    '''The AST node for defining a module.'''

    ATTR_EXPLICIT_FIFO = 'explicit_fifo'
    ATTR_WAIT_UNTIL = 'wait_until'
    ATTR_NO_ARBITER = 'no_arbiter'

    def __init__(self):
        self.name = type(self).__name__
        if self.name not in ['Driver', 'Testbench']:
            self.name = self.name + hex(id(self))[-5:-1]
        self.body = None
        self._restore_ports = {}
        self.binds = 0
        self.attrs = set()

    @property
    def ports(self):
        '''The helper function to get all the ports in the module.'''
        return [v for _, v in self.__dict__.items() if isinstance(v, Port)]

    @ir_builder(node_type='expr')
    def async_called(self, **kwargs):
        '''The frontend API for creating an async call operation to this `self` module.'''
        bind = self.bind(**kwargs)
        return AsyncCall(bind)

    @ir_builder(node_type='expr')
    def bind(self, **kwargs):
        '''The frontend API for creating a bind operation to this `self` module.'''
        bound = Bind(self, **kwargs)
        self.binds += 1
        return bound

    def as_operand(self):
        '''Dump the module as a right-hand side reference.'''
        return self.name

    def __repr__(self):
        ports = '\n    '.join(repr(v) for v in self.ports)
        if ports:
            ports = f'{{\n    {ports}\n  }} '
        Singleton.repr_ident = 2
        body = self.body.__repr__()
        return f'  module {self.name} {ports}{{\n{body}\n  }}'

    @property
    def implicit_fifo(self):
        '''The helper function to get the implicit FIFO setting.'''
        return self.ATTR_EXPLICIT_FIFO not in self.attrs

    @implicit_fifo.setter
    def implicit_fifo(self, value):
        '''The helper function to set the implicit FIFO setting.'''
        if value:
            self.attrs.discard(self.ATTR_EXPLICIT_FIFO)
        else:
            self.attrs.add(self.ATTR_EXPLICIT_FIFO)

    def _implicit_pop(self):
        if self.implicit_fifo:
            for port in self.ports:
                self._restore_ports[port.name] = port
                setattr(self, port.name, port.pop())

    def _flip_restore(self):
        '''The helper function swaps FIFO.pop's and the original ports.'''
        if self.implicit_fifo:
            to_restore = list(self._restore_ports.items())
            for k, v in to_restore:
                self._restore_ports[k] = getattr(self, k)
                setattr(self, k, v)

class Port:
    '''The AST node for defining a port in modules.'''

    def __init__(self, dtype: DType):
        assert isinstance(dtype, DType)
        self.dtype = dtype
        self.name = self.module = None

    @ir_builder(node_type='expr')
    def valid(self):
        '''The frontend API for creating a FIFO.valid operation.'''
        return FIFOField(FIFOField.FIFO_VALID, self)

    @ir_builder(node_type='expr')
    def peek(self):
        '''The frontend API for creating a FIFO.peek operation.'''
        return FIFOField(FIFOField.FIFO_PEEK, self)

    @ir_builder(node_type='expr')
    def pop(self):
        '''The frontend API for creating a pop operation.'''
        return FIFOPop(self)

    @ir_builder(node_type='expr')
    def push(self, v):
        '''The frontend API for creating a push operation.'''
        return FIFOPush(self, v)

    def __repr__(self):
        return f'{self.name}: port<{self.dtype}>'

    def as_operand(self):
        '''Dump the port as a right-hand side reference.'''
        return f'{self.module.name}.{self.name}'

@decorator
#pylint: disable=keyword-arg-before-vararg
def combinational(func, implicit_fifo=True, *args, **kwargs):
    '''A decorator for marking a function as combinational logic description.'''
    module_self = args[0]
    assert isinstance(module_self, Module)
    module_self.implicit_fifo = implicit_fifo
    module_self.body = Block(Block.MODULE_ROOT)
    Singleton.builder.insert_point['expr'] = module_self.body.body
    Singleton.builder.cur_module = module_self
    Singleton.builder.builder_func = func

    #pylint: disable=protected-access
    module_self._implicit_pop()

    res = func(*args, **kwargs)

    module_self._flip_restore()

    Singleton.builder.cleanup_symtab()
    Singleton.builder.cur_module = None
    Singleton.builder.insert_point['expr'] = None
    return res
