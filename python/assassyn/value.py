from .builder import ir_builder

class Value(object):

    @ir_builder(node_type='expr')
    def __add__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.ADD, self, other)

    @ir_builder(node_type='expr')
    def __sub__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.SUB, self, other)

    @ir_builder(node_type='expr')
    def __mul__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.MUL, self, other)

    @ir_builder(node_type='expr')
    def __ror__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_OR, self, other)

    @ir_builder(node_type='expr')
    def __rxor__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_XOR, self, other)

    @ir_builder(node_type='expr')
    def __rand__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_AND, self, other)

    @ir_builder(node_type='expr')
    def __getitem__(self, x):
        from .expr import Slice
        if isinstance(x, slice):
            return Slice(self, int(x.start), int(x.stop))
        else:
            assert False, "Expecting a slice object"

    @ir_builder(node_type='expr')
    def __lt__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.ILT, self, other)

    @ir_builder(node_type='expr')
    def __gt__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.IGT, self, other)

    @ir_builder(node_type='expr')
    def __le__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.ILE, self, other)

    @ir_builder(node_type='expr')
    def __ge__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.IGE, self, other)

    @ir_builder(node_type='expr')
    def bitcast(self, dtype):
        from .expr import Cast
        return Cast(Cast.BITCAST, self, dtype)

    @ir_builder(node_type='expr')
    def concat(self, other):
        from .expr import Concat
        return Concat(self, other)
