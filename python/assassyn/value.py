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
    def __eq__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.EQ, self, other)

    @ir_builder(node_type='expr')
    def __ne__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.NEQ, self, other)

    @ir_builder(node_type='expr')
    def __mod__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.MOD, self, other)

    @ir_builder(node_type='expr')
    def __invert__(self):
        from .expr import UnaryOp
        return UnaryOp(UnaryOp.FLIP, self)

    @ir_builder(node_type='expr')
    def __lshift__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.SHL, self, other)

    @ir_builder(node_type='expr')
    def __rshift__(self, other):
        from .expr import BinaryOp
        return BinaryOp(BinaryOp.SHR, self, other)

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

    @ir_builder(node_type='expr')
    def select(self, true_value, false_value):
        '''The frontend API to create a select operation'''
        from .expr import Select
        return Select(Select.SELECT, self, true_value, false_value)

    @ir_builder(node_type='expr')
    def select1hot(self, *args):
        '''The frontend API to create a select1hot operation'''
        from .expr import Select1Hot
        return Select1Hot(Select1Hot.SELECT_1HOT, self, args)

    @ir_builder(node_type='expr')
    def valid(self):
        '''The frontend API to check if this value is valid.
        NOTE: This operation is only usable in downstream modules.'''
        from .expr import PureInstrinsic
        return PureInstrinsic(PureInstrinsic.VALUE_VALID, self)

class RecordValue:
    '''The value class for the record type. Remember, this is a right-value object, so each
    field of this record is immutable!'''

    def __init__(self, dtype, **kwargs):

        from .dtype import Record
        assert isinstance(dtype, Record), "Expecting a Record type!"

        ordered_values = []
        fields_set = set(dtype.fields.keys())
        # TODO(@were): Check all the values are already in bits type
        for name, value in kwargs.items():
            assert name in dtype.fields, f'Field {name} not found in {dtype.fields} of this Record'
            fields_set.remove(name)
            _, attr_slice = dtype.fields[name]
            ordered_values.append((attr_slice, value))

        assert not fields_set, f'Fields are not fully initialized, missing: {fields_set}'
        ordered_values.sort(key=lambda x: x[0].start)

        from .expr import concat
        payload = concat(*[v for _, v in ordered_values])

        self.payload = payload
        self.dtype = dtype

    def as_operand(self):
        '''Return the payload as an operand'''
        return self.payload.as_operand()

    def __repr__(self):
        return f'RecordValue({self.dtype}, {self.payload})'

    # A Python TIP: __getattr__ is a "fallback" method, when "name" attribute is not found in the
    # self object. However, __getattribute__ is a "hook" method, which is called when every a.b
    # field access is made. If you do anything like self.a in __getattribute__, it will cause a
    # infinite recursion.
    def __getattr__(self, name):
        return self.dtype.attributize(self.payload, name)
