'''Arithmetic and logical operations'''

#pylint: disable=cyclic-import

from __future__ import annotations

import typing

from ..value import Value
from .expr import Expr

if typing.TYPE_CHECKING:
    from ..dtype import DType


class BinaryOp(Expr):
    '''The class for binary operations'''

    lhs: Value  # Left-hand side operand
    rhs: Value  # Right-hand side operand

    # Binary operations
    ADD         = 200
    SUB         = 201
    MUL         = 202
    DIV         = 203
    MOD         = 204
    BITWISE_AND = 206
    BITWISE_OR  = 207
    BITWISE_XOR = 208
    ILT         = 209
    IGT         = 210
    ILE         = 211
    IGE         = 212
    EQ          = 213
    SHL         = 214
    SHR         = 215
    NEQ         = 216

    OPERATORS = {
      ADD: '+',
      SUB: '-',
      MUL: '*',
      DIV: '/',
      MOD: '%',

      ILT: '<',
      IGT: '>',
      ILE: '<=',
      IGE: '>=',
      EQ:  '==',
      NEQ: '!=',

      BITWISE_AND: '&',
      BITWISE_OR:  '|',
      BITWISE_XOR: '^',

      SHL: '<<',
      SHR: '>>',
    }

    def __init__(self, opcode, lhs, rhs):
        assert isinstance(lhs, Value), f'{type(lhs)} is not a Value!'
        assert isinstance(rhs, Value), f'{type(rhs)} is not a Value!'
        super().__init__(opcode, [lhs, rhs])

    @property
    def lhs(self):
        '''Get the left-hand side operand'''
        return self._operands[0]

    @property
    def rhs(self):
        '''Get the right-hand side operand'''
        return self._operands[1]

    @property
    def dtype(self):
        '''Get the data type of this operation'''
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits
        if self.opcode in [BinaryOp.ADD]:
            # TODO(@were): Make this bits + 1
            bits = max(self.lhs.dtype.bits, self.rhs.dtype.bits)
            tyclass = self.lhs.dtype.__class__
            return tyclass(bits)
        if self.opcode in [BinaryOp.SUB, BinaryOp.DIV, BinaryOp.MOD]:
            return type(self.lhs.dtype)(self.lhs.dtype.bits)
        if self.opcode in [BinaryOp.MUL]:
            bits = self.lhs.dtype.bits + self.rhs.dtype.bits
            tyclass = self.lhs.dtype.__class__
            return tyclass(bits)
        if self.opcode in [BinaryOp.SHL, BinaryOp.SHR]:
            return Bits(self.lhs.dtype.bits)
        if self.opcode in [BinaryOp.ILT, BinaryOp.IGT, BinaryOp.ILE, BinaryOp.IGE,
                           BinaryOp.EQ, BinaryOp.NEQ]:
            return Bits(1)
        if self.opcode in [BinaryOp.BITWISE_AND, BinaryOp.BITWISE_OR, BinaryOp.BITWISE_XOR]:
            return Bits(max(self.lhs.dtype.bits, self.rhs.dtype.bits))
        raise NotImplementedError(f'Unsupported binary operation {self.opcode}')

    def __repr__(self):
        lval = self.as_operand()
        lhs = self.lhs.as_operand()
        rhs = self.rhs.as_operand()
        op = self.OPERATORS[self.opcode]
        return f'{lval} = {lhs} {op} {rhs}'

    def is_computational(self):
        '''Check if this operation is computational'''
        return self.opcode in [BinaryOp.ADD, BinaryOp.SUB, BinaryOp.MUL, BinaryOp.DIV,
                               BinaryOp.MOD]

    def is_comparative(self):
        '''Check if this operation is comparative'''
        return self.opcode in [BinaryOp.ILT, BinaryOp.IGT, BinaryOp.ILE, BinaryOp.IGE,
                               BinaryOp.EQ, BinaryOp.NEQ]

class UnaryOp(Expr):
    '''The class for unary operations'''

    # Unary operations
    NEG  = 100
    FLIP = 101

    OPERATORS = {
        NEG: '-',
        FLIP: '!',
    }

    def __init__(self, opcode, x):
        super().__init__(opcode, [x])

    @property
    def x(self) -> Value:
        '''Get the operand of this unary operation'''
        return self._operands[0]

    @property
    def dtype(self) -> DType:
        '''Get the data type of this unary operation'''
        # pylint: disable=import-outside-toplevel
        from ..dtype import Bits
        return Bits(self.x.dtype.bits)

    def __repr__(self):
        return f'{self.as_operand()} = {self.OPERATORS[self.opcode]}{self.x.as_operand()}'
