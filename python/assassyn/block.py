'''The module for the block AST node related implementations.'''

from .builder import ir_builder, Singleton

class Block:
    '''The base node of a block.'''

    MODULE_ROOT = 0
    CONDITIONAL = 1
    CYCLE       = 2

    def __init__(self, kind: int):
        self.kind = kind
        self._body = []
        self._restore = None
        self.parent = None

    def __repr__(self):
        Singleton.repr_ident += 2
        ident = ' ' * Singleton.repr_ident
        body = ident + ('\n' + ident).join(repr(elem) for elem in self.iter())
        Singleton.repr_ident -= 2
        return body

    def as_operand(self):
        '''Dump the block as an operand.'''
        return f'_{hex(id(self))[-5:-1]}'

    def insert(self, x, elem):
        '''Insert an instruction at the specified position.'''
        self._body.insert(x, elem)

    def iter(self):
        '''Iterate over the block.'''
        yield from self._body

    def __enter__(self):
        '''Designate the scope of entering the block.'''
        assert self._restore is None, "A block cannot be used twice!"
        parent = Singleton.builder.get_current_block()
        if parent is None:
            parent = Singleton.builder.get_current_module()
        assert parent is not None
        self.parent = parent
        self._restore = Singleton.builder.insert_point['expr']
        Singleton.builder.insert_point['expr'] = self._body
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        '''Designate the scope of exiting the block.'''
        Singleton.builder.insert_point['expr'] = self._restore

class CondBlock(Block):
    '''The inherited class of the block for conditional block.'''

    def __init__(self, cond):
        super().__init__(Block.CONDITIONAL)
        self.cond = cond

    def __repr__(self):
        ident = Singleton.repr_ident * ' '
        res = f'when {self.cond.as_operand()} {{\n'
        res = res + super().__repr__()
        res = res + f'\n{ident}}}'
        return res

class CycledBlock(Block):
    '''The inherited class of the block for cycled block used for testbench generation.'''

    def __init__(self, cycle: int):
        super().__init__(Block.CYCLE)
        self.cycle = cycle

    def __repr__(self):
        ident = Singleton.repr_ident * ' '
        res = f'cycle {self.cycle} {{\n'
        res = res + super().__repr__()
        res = res + f'\n{ident}}}'
        return res

@ir_builder(node_type='expr')
def Condition(cond): # pylint: disable=invalid-name
    #pylint: disable=import-outside-toplevel
    '''Frontend API for creating a conditional block.'''
    from .value import Value
    assert isinstance(cond, Value)
    return CondBlock(cond)

@ir_builder(node_type='expr')
def Cycle(cycle: int): # pylint: disable=invalid-name
    '''Frontend API for creating a cycled block.'''
    assert isinstance(cycle, int)
    return CycledBlock(cycle)
