"""The base node module for the overloaded frontend."""

from __future__ import annotations

from abc import ABC, abstractmethod

from ..builder import ir_builder

#pylint: disable=import-outside-toplevel,cyclic-import

class Value(ABC):
    '''Base class for overloading arithmetic operations in the frontend'''

    name: str | None  # Name for this value (used for IR generation and debugging)

    @property
    @abstractmethod
    def dtype(self):
        '''Abstract property for data type. All Value subclasses must implement this.'''

    @ir_builder
    def __add__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.ADD, self, other)

    @ir_builder
    def __sub__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.SUB, self, other)

    @ir_builder
    def __mul__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.MUL, self, other)

    @ir_builder
    def __or__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_OR, self, other)

    @ir_builder
    def __xor__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_XOR, self, other)

    @ir_builder
    def __and__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.BITWISE_AND, self, other)

    @ir_builder
    def __getitem__(self, x):
        from .array import Slice
        if isinstance(x, slice):
            return Slice(self, int(x.start), int(x.stop))
        assert False, "Expecting a slice object"

    @ir_builder
    def __lt__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.ILT, self, other)

    @ir_builder
    def __gt__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.IGT, self, other)

    @ir_builder
    def __le__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.ILE, self, other)

    @ir_builder
    def __ge__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.IGE, self, other)

    @ir_builder
    def __eq__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.EQ, self, other)

    @ir_builder
    def __ne__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.NEQ, self, other)

    @ir_builder
    def __mod__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.MOD, self, other)

    @ir_builder
    def __invert__(self):
        from .expr import UnaryOp
        return UnaryOp(UnaryOp.FLIP, self)

    @ir_builder
    def __lshift__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.SHL, self, other)

    @ir_builder
    def __rshift__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.SHR, self, other)

    def __hash__(self):
        return id(self)

    # This is a pitfall of developing the frontend. This optional method is served as a "ir_builder"
    # API, but it should not be annotated with this decorator. It calls the "select" method, and
    # the called "select" method will insert the generated Select node into the AST. It we annotate
    # this method here, the generated node will be inserted into the AST twice.
    def optional(self, default, predicate=None):
        '''The frontend API to create an optional value with default'''
        if predicate is None:
            predicate = self.valid()
        assert isinstance(predicate, Value), "Expecting a Value object"
        return predicate.select(self, default)

    @ir_builder
    def bitcast(self, dtype):
        '''The frontend API to create a bitcast operation'''
        from .expr import Cast
        return Cast(Cast.BITCAST, self, dtype)

    @ir_builder
    def zext(self, dtype):
        '''The frontend API to create a zero-extend operation'''
        from .expr import Cast
        return Cast(Cast.ZEXT, self, dtype)

    @ir_builder
    def sext(self, dtype):
        '''The frontend API to create a sign-extend operation'''
        from .expr import Cast
        return Cast(Cast.SEXT, self, dtype)

    @ir_builder
    def concat(self, other):
        #pylint: disable=no-member
        '''The frontend API to create a bitwise-concat operation'''
        from .expr import Concat
        return Concat(self, other)

    @ir_builder
    def select(self, true_value, false_value):
        '''The frontend API to create a select operation'''
        from .expr import Select
        return Select(Select.SELECT, self, true_value, false_value)

    def case(self, cases: dict['Value', 'Value']):
        '''The frontend API to create a case operation'''
        assert None in cases, "Expecting a default case"
        res = cases[None]
        for k, v in cases.items():
            if k is None:
                continue
            assert isinstance(k, Value), "Expecting a Value object for key"
            assert isinstance(v, Value), "Expecting a Value object for value"
            res = (self == k).select(v, res)
        return res

    @ir_builder
    def select1hot(self, *args):
        '''The frontend API to create a select1hot operation'''
        from .expr import Select1Hot
        return Select1Hot(Select1Hot.SELECT_1HOT, self, args)

    @ir_builder
    def valid(self):
        '''The frontend API to check if this value is valid.
        NOTE: This operation is only usable in downstream modules.'''
        from .expr.intrinsic import PureIntrinsic
        return PureIntrinsic(PureIntrinsic.VALUE_VALID, self)
