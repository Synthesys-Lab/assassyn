from .builder import ir_builder, Singleton

class Block(object):

    MODULE_ROOT = 0
    CONDITIONAL = 1

    def __init__(self, kind: int):
        self.kind = kind
        self.body = []

    def __repr__(self):
        Singleton.repr_ident += 2
        ident = ' ' * Singleton.repr_ident
        body = ident + ('\n' + ident).join(repr(elem) for elem in self.body)
        Singleton.repr_ident -= 2
        return body

    def as_operand(self):
        return f'_{hex(id(self))[-5:-1]}'

class CondBlock(Block):

    def __init__(self, cond):
        super().__init__(Block.CONDITIONAL)
        self.cond = cond
        self.restore = None

    def __enter__(self):
        assert self.restore is None, "A block cannot be used twice!"
        self.restore = Singleton.builder.insert_point['expr']
        Singleton.builder.insert_point['expr'] = self.body
        return self

    def __exit__(self, exc_type, exc_value, traceback):
        Singleton.builder.insert_point['expr'] = self.restore

    def __repr__(self):
        ident = Singleton.repr_ident * ' '
        res = f'when {self.cond.as_operand()} {{\n'
        res = res + super().__repr__()
        res = res + f'\n{ident}}}'
        return res 


@ir_builder(node_type='expr')
def Condition(cond):
    return CondBlock(cond)
