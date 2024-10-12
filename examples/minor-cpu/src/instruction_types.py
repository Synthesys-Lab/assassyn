from assassyn.frontend import *
from opcodes import *
class InstType:

    FIELDS = [
        ((0, 6), 'opcode', Bits),
        ((7, 11), 'rd', Bits),
        ((15, 19), 'rs1', Bits),
        ((20, 24), 'rs2', Bits),
        ((12, 14), 'funct3', Bits),
        ((25, 31), 'funct7', Bits),
    ]

    def __init__(self, rd, rs1, rs2, funct3, funct7, fields):
        self.fields = fields.copy()
        for cond, entry in zip([True, rd, rs1, rs2, funct3, funct7], self.FIELDS):
            key, field, ty = entry
            setattr(self, f'has_{field}', cond)
            if cond:
                self.fields[key] = (field, ty)
        self.dtype = Record(self.fields)
        self.value = None

    def view(self):
        return self.dtype.view(self.value)

class RInst(InstType):

    PREFIX = 'r'

    def __init__(self, value):
        super().__init__(True, True, True, True, True, {})
        self.value = value

    def imm(self, pad):
        return None

class IInst(InstType):

    PREFIX = 'i'

    def __init__(self, value):
        super().__init__(True, True, False, True, False, { (20, 31): ('imm', Bits) })
        self.value = value

    def imm(self, pad):
        raw = self.view().imm
        if pad:
            raw = concat(Bits(20)(0), raw)
        return raw

class SInst(InstType):

    PREFIX = 's'

    def __init__(self, value):
        fields = { (25, 31): ('imm11_5', Bits), (7, 11): ('imm4_0', Bits) }
        super().__init__(False, True, True, True, False, fields)
        self.value = value

    def imm(self, pad):
        imm = self.view().imm11_5.concat(self.view().imm4_0)
        if pad:
            imm = concat(Bits(20)(0), imm)
        return imm

class UInst(InstType):

    PREFIX = 'u'

    def __init__(self, value):
        super().__init__(True, False, False, False, False, { (12, 31): ('imm', Bits) })
        self.value = value

    def imm(self, pad):
        raw = self.view().imm
        if pad:
            raw = concat(Bits(12)(0), raw)
        return raw

class BInst(InstType):

    PREFIX = 'b'

    def __init__(self, value):
        fields = {
            (7, 7): ('imm11', Bits),
            (8, 11): ('imm4_1', Bits),
            (25, 30): ('imm10_5', Bits),
            (31, 31): ('imm12', Bits),
        }
        super().__init__(False, True, True, True, False, fields)
        self.value = value

    def imm(self, pad):
        imm = concat(self.view().imm12, self.view().imm11, self.view().imm10_5, self.view().imm4_1)
        imm = imm.concat(Bits(1)(0))
        if pad:
            imm = concat(Bits(19)(0), imm)
        return imm

supported_opcodes = [
  # mn,     opcode,    type
  ('lui'   , 0b0110111, UInst),
  ('addi'  , 0b0010011, IInst),
  ('add'   , 0b0110011, RInst),
  ('lw'    , 0b0000011, IInst),
  ('bne'   , 0b1100011, BInst),
  ('ret'   , 0b1101111, UInst),
  ('ebreak', 0b1110011, IInst),
]

deocder_signals = Record(
  memory_read=Bits(1),
  invoke_adder=Bits(1),
  is_branch=Bits(1),
  rs1_reg=Bits(5),
  rs1_valid=Bits(1),
  rs2_reg=Bits(5),
  rs2_valid=Bits(1),
  rd_reg=Bits(5),
  rd_valid=Bits(1),
  imm_valid=Bits(1),
  imm_value=Bits(32),
)

supported_types = [RInst, IInst, SInst, BInst, UInst]
