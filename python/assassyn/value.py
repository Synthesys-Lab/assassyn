'''The base node module for the overloaded frontend'''

from .builder import ir_builder

#pylint: disable=import-outside-toplevel

class Value:
    '''Base class for overloading arithmetic operations in the frontend'''

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
    def __or__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_OR, self, other)

    @ir_builder(node_type='expr')
    def __xor__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_XOR, self, other)

    @ir_builder(node_type='expr')
    def __and__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_AND, self, other)

    @ir_builder(node_type='expr')
    def __getitem__(self, x):
        from .expr import Slice
        if isinstance(x, slice):
            return Slice(self, int(x.start), int(x.stop))
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
    def __invert__(self):
        from .expr import UnaryOp
        return UnaryOp(UnaryOp.FLIP, self)

    @ir_builder(node_type='expr')
    def bitcast(self, dtype):
        '''The frontend API to create a bitcast operation'''
        from .expr import Cast
        return Cast(Cast.BITCAST, self, dtype)

    @ir_builder(node_type='expr')
    def zext(self, dtype):
        '''The frontend API to create a zero-extend operation'''
        from .expr import Cast
        return Cast(Cast.ZEXT, self, dtype)

    @ir_builder(node_type='expr')
    def sext(self, dtype):
        '''The frontend API to create a sign-extend operation'''
        from .expr import Cast
        return Cast(Cast.SEXT, self, dtype)

    @ir_builder(node_type='expr')
    def concat(self, other):
        '''The frontend API to create a bitwise-concat operation'''
        from .expr import Concat
        return Concat(self, other)
