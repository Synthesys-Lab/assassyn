'''The module AST implementation.'''

from decorator import decorator

from ..builder import Singleton, ir_builder
from ..dtype import DType
from ..block import Block
from ..expr import Bind, FIFOPop, PureInstrinsic, FIFOPush, AsyncCall
from ..expr.intrinsic import _wait_until
from .base import ModuleBase, name_ports_of_module

@decorator
def constructor(func, *args, **kwargs):
    '''A decorator for marking a function as a constructor of a module.'''
    builder = Singleton.builder
    module_self = args[0]
    builder.insert_point['module'].append(module_self)
    func(*args, **kwargs)
    for key in ['body', '_pop_cache', '_wait_until', '_finalized']:
        assert hasattr(module_self, key), 'Did you forget to call `super().__init__?`'
    name_ports_of_module(module_self, Port)

def _reserved_module_name(name):
    return name in ['Driver', 'Testbench']

@decorator
def wait_until(func, *args, **kwargs):
    '''A decorator for marking a module with wait-until logic.'''
    #pylint: disable=protected-access
    module_self = args[0]
    assert isinstance(module_self, Module)
    Singleton.builder.cur_module = module_self
    module_self._attrs[Module.ATTR_TIMING] = Timing(Timing.BACKPRESSURE)

    restored = module_self.implicit_restore()
    restore = Singleton.builder.insert_point['expr']
    Singleton.builder.insert_point['expr'] = module_self._wait_until
    cond = func(*args, **kwargs)
    _wait_until(cond)
    if restored:
        module_self.implicit_pop()

    Singleton.builder.insert_point['expr'] = restore
    Singleton.builder.cur_module = None

    return module_self._wait_until

#pylint: disable=too-few-public-methods
class Timing:
    '''The enum class for the timing policy of a module.'''
    UNDEFINED = 0
    SYSTOLIC = 1
    BACKPRESSURE = 2

    def __init__(self, ty):
        self.ty = ty

    def __repr__(self):
        return ['undefined', 'systolic', 'backpressure'][self.ty]

class Module(ModuleBase):
    '''The AST node for defining a module.'''

    ATTR_EXPLICIT_FIFO = 0
    ATTR_DISABLE_ARBITER = 1
    ATTR_MEMORY = 2
    ATTR_TIMING = 3

    MODULE_ATTR_STR = {
      ATTR_EXPLICIT_FIFO: 'explicit_fifo',
      ATTR_DISABLE_ARBITER: 'no_arbiter',
      ATTR_MEMORY: 'memory',
      ATTR_TIMING: 'timing',
    }

    def __init__(
            self,
            explicit_fifo=False,
            timing=Timing.UNDEFINED,
            disable_arbiter_rewrite=False):
        '''Construct the module with the given attributes.
        
        Args:
          - explicit_fifo(bool): If this module explicitly pops FIFO values.
          - timing(Timing): The timing policy of this module.
          - disable_arbiter_rewrite(bool): When there are multiple callers, if this module
          should be rewritten by the compiler.
        '''
        super().__init__()
        self.body = None
        self._pop_cache = {}
        self._wait_until = []
        self._finalized = False
        self.name = type(self).__name__
        self._attrs = {}
        self.parse_attrs(explicit_fifo, timing, disable_arbiter_rewrite)
        if not _reserved_module_name(self.name):
            self.name = self.name + self.as_operand()

    def validate_all_ports(self):
        '''A syntactic sugar for checking the validity of all the ports in this module.'''
        valid = None
        for port in self.ports:
            valid = port.valid() if valid is None else valid & port.valid()
        return valid

    @property
    def finalized(self):
        '''The helper function to finalize the module.'''
        return self._finalized

    @finalized.setter
    def finalized(self, value):
        if value:
            self._finalized = True
            concat = self._wait_until + list(self.fifo_pops.values()) + list(self.body.iter())
            #pylint: disable=protected-access
            self.body._body = concat
        elif self._finalized:
            assert False, 'Finalization cannot be reverted!'

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
        return bound

    def __repr__(self):
        ports = '\n    '.join(repr(v) for v in self.ports)
        if ports:
            ports = f'{{\n    {ports}\n  }} '
        attrs = ', '.join(f'{Module.MODULE_ATTR_STR[i]}: {j}' for i, j in self._attrs.items())
        attrs = f'#[{attrs}] ' if attrs else ''
        var_id = self.as_operand()
        if self.finalized:
            Singleton.repr_ident = 2
            body = self.body.__repr__()
            return f'''  {attrs}
  {var_id} = module {self.name} {ports}{{
{body}
  }}
'''
        Singleton.repr_ident = 4
        body = self.body.__repr__() if self.body is not None else ''
        precond = '\n      '.join(repr(v) for v in self._wait_until)
        if precond:
            precond = f'''
     "wait_until": {{
       {precond}
     }}'''
        pops = '\n      '.join(repr(v) for v in self.fifo_pops.values())
        if pops:
            pops = f'''
    "pops": {{
      {pops}
    }}'''
        return f'''  {attrs}
  {var_id} = module {self.name} {ports}{{{precond}{pops}
    "body": {{
{body}
    }}
  }}
'''

    @property
    def fifo_pops(self):
        '''Handle implicit FIFO pop and cache them in a dict.'''
        if self.is_explicit_fifo:
            return {}
        if not self._pop_cache:
            self._pop_cache = { port: FIFOPop(port) for port in self.ports }
        return self._pop_cache

    def implicit_pop(self):
        '''Implicitly replace all the ports with FIFO.pop operations.'''
        if not self.is_explicit_fifo:
            cache = self.fifo_pops
            for k, v in self.__dict__.items():
                if isinstance(v, Port):
                    setattr(self, k, cache[v])

    def implicit_restore(self):
        '''Implicitly restore all the FIFO.pop back to FIFOs.'''
        restored = False
        if not self.is_explicit_fifo:
            for k, v in self.__dict__.items():
                if isinstance(v, FIFOPop):
                    setattr(self, k, v.fifo)
                    restored = True
        return restored

    def parse_attrs(self, is_explicit_fifo, timing, disable_arbiter_rewrite):
        '''The helper function to parse the attributes.'''
        self._attrs[Module.ATTR_EXPLICIT_FIFO] = is_explicit_fifo
        self._attrs[Module.ATTR_TIMING] = Timing(timing)
        self._attrs[Module.ATTR_DISABLE_ARBITER] = disable_arbiter_rewrite

    @property
    def is_systolic(self):
        '''The helper function to get if this module is systolic.'''
        value = self._attrs.get(Module.ATTR_TIMING, Timing(Timing.UNDEFINED)).ty
        return value == Timing.SYSTOLIC

    @property
    def disable_arbiter_rewrite(self):
        '''The helper function to get the no-arbiter setting.'''
        return self._attrs.get(Module.ATTR_DISABLE_ARBITER, False)

    @property
    def is_explicit_fifo(self):
        '''The helper function to get the implicit FIFO setting.'''
        return self._attrs.get(Module.ATTR_EXPLICIT_FIFO, False)


class Port:
    '''The AST node for defining a port in modules.'''

    def __init__(self, dtype: DType):
        assert isinstance(dtype, DType)
        self.dtype = dtype
        self.name = self.module = None

    @ir_builder(node_type='expr')
    def valid(self):
        '''The frontend API for creating a FIFO.valid operation.'''
        return PureInstrinsic(PureInstrinsic.FIFO_VALID, self)

    @ir_builder(node_type='expr')
    def peek(self):
        '''The frontend API for creating a FIFO.peek operation.'''
        return PureInstrinsic(PureInstrinsic.FIFO_PEEK, self)

    @ir_builder(node_type='expr')
    def pop(self):
        '''The frontend API for creating a pop operation.'''
        return FIFOPop(self)

    @ir_builder(node_type='expr')
    def push(self, v):
        '''The frontend API for creating a push operation.'''
        return FIFOPush(self, v)

    def __repr__(self):
        return f'{self.name}: Port<{self.dtype}>'

    def as_operand(self):
        '''Dump the port as a right-hand side reference.'''
        return f'{self.module.as_operand()}.{self.name}'

@decorator
#pylint: disable=keyword-arg-before-vararg
def combinational(
        func,
        *args,
        **kwargs):
    '''A decorator for marking a function as combinational logic description.'''
    module_self = args[0]
    assert isinstance(module_self, Module)
    Singleton.builder.cur_module = module_self
    Singleton.builder.builder_func = func
    module_self.body = Block(Block.MODULE_ROOT)
    # TODO(@were): Make implicit pop more robust.
    with module_self.body:
        module_self.implicit_pop()
        res = func(*args, **kwargs)
        module_self.implicit_restore()
    Singleton.builder.cleanup_symtab()
    Singleton.builder.cur_module = None
    return res
