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

    def __init__(self, rd, rs1, rs2, funct3, funct7, fields, value):
        self.fields = fields.copy()
        for cond, entry in zip([True, rd, rs1, rs2, funct3, funct7], self.FIELDS):
            key, field, ty = entry
            setattr(self, f'has_{field}', cond)
            if cond:
                self.fields[key] = (field, ty)
        self.dtype = Record(self.fields)
        self.value = value

    def view(self):
        return self.dtype.view(self.value)

class InstSignal:

    def __init__(self, eq, alu, cond=None):
        self.eq = eq

        self.alu = Bits(RV32I_ALU.CNT)(0)
        if alu is not None:
            self.alu = Bits(RV32I_ALU.CNT)(1 << alu)

        self.cond = Bits(RV32I_ALU.CNT)(1)
        self.flip = Bits(1)(0)
        if cond is not None:
            pred, flip = cond
            self.cond = Bits(RV32I_ALU.CNT)(1 << pred)
            self.flip = Bits(1)(flip)

class RInst(InstType):

    PREFIX = 'r'

    def __init__(self, value):
        super().__init__(True, True, True, True, True, {}, value)

    def decode(self, opcode, funct3, funct7, alu):
        view = self.view()
        opcode = view.opcode == Bits(7)(opcode)
        funct3 = view.funct3 == Bits(3)(funct3)
        funct7 = view.funct7 == Bits(7)(funct7)
        eq = opcode & funct3 & funct7
        return InstSignal(eq, alu)


    def imm(self, pad):
        return None

class IInst(InstType):

    PREFIX = 'i'

    def __init__(self, value):
        super().__init__(True, True, False, True, False, { (20, 31): ('imm', Bits) }, value)

    def imm(self, pad):
        raw = self.view().imm
        if pad:
            signal = raw[11:11]
            signal = signal.select(Bits(20)(0xfffff), Bits(20)(0))
            raw = concat(signal, raw)
        return raw

    def decode(self, opcode, funct3, alu, cond):
        view = self.view()
        opcode = view.opcode == Bits(7)(opcode)
        funct3 = view.funct3 == Bits(3)(funct3)
        eq = opcode & funct3
        return InstSignal(eq, alu, cond=cond)

class SInst(InstType):

    PREFIX = 's'

    def __init__(self, value):
        fields = { (25, 31): ('imm11_5', Bits), (7, 11): ('imm4_0', Bits) }
        super().__init__(False, True, True, True, False, fields, value)

    def decode(self, *args):
        raise NotImplementedError

    def imm(self, pad):
        raise NotImplementedError
        imm = self.view().imm11_5.concat(self.view().imm4_0)
        if pad:
            imm = concat(Bits(20)(0), imm)
        return imm

class UInst(InstType):

    PREFIX = 'u'

    def __init__(self, value):
        super().__init__(True, False, False, False, False, { (12, 31): ('imm', Bits) }, value)

    def decode(self, opcode, alu):
        view = self.view()
        eq = view.opcode == Bits(7)(opcode)
        return InstSignal(eq, alu)

    def imm(self, pad):
        raw = self.view().imm
        if pad:
            raw = concat(Bits(12)(0), raw)
        return raw

class JInst(InstType):

    PREFIX = 'j'

    def __init__(self, value):
        fields = {
            (12, 19): ('imm19_12', Bits),
            (20, 20): ('imm11', Bits),
            (21, 30): ('imm10_1', Bits),
            (31, 31): ('imm20', Bits),
        }
        super().__init__(True, False, False, False, False, fields, value)

    def decode(self, opcode, alu):
        view = self.view()
        eq = view.opcode == Bits(7)(opcode)
        return InstSignal(eq, alu)

    def imm(self, pad):
        view = self.view()
        imm = concat(view.imm20, view.imm19_12, view.imm11, view.imm10_1, Bits(1)(0))
        if pad:
            signal = imm[20:20]
            signal = signal.select(Bits(11)(0x7ff), Bits(11)(0))
            imm = concat(signal, imm)
        return imm

class BInst(InstType):

    PREFIX = 'b'

    def __init__(self, value):
        fields = {
            (7, 7): ('imm11', Bits),
            (8, 11): ('imm4_1', Bits),
            (25, 30): ('imm10_5', Bits),
            (31, 31): ('imm12', Bits),
        }
        super().__init__(False, True, True, True, False, fields, value)

    def decode(self, opcode, funct3, cmp, flip):
        view = self.view()
        opcode = view.opcode == Bits(7)(opcode)
        funct3 = view.funct3 == Bits(3)(funct3)
        eq = opcode & funct3
        return InstSignal(eq, RV32I_ALU.ALU_ADD, cond=(cmp, flip))

    def imm(self, pad):
        imm = concat(self.view().imm12, self.view().imm11, self.view().imm10_5, self.view().imm4_1)
        imm = imm.concat(Bits(1)(0))
        if pad:
            signal = imm[12:12]
            signal = signal.select(Bits(19)(0x7ffff), Bits(19)(0))
            imm = concat(signal, imm)
        return imm

class RV32I_ALU:
    CNT = 16

    ALU_ADD = 0
    ALU_SUB = 1
    ALU_XOR = 2
    ALU_OR = 3
    ALU_AND = 4
    ALU_SLL = 5
    ALU_SRL = 6
    ALU_SRA = 7
    ALU_CMP_EQ = 8
    ALU_CMP_LT = 9
    ALU_CMP_LTU = 10
    # Always true.
    ALU_TRUE = 11

supported_opcodes = [
  # mn,     opcode,      operator
  ('jal'   , (0b1101111, RV32I_ALU.ALU_ADD), JInst),

  ('lui'   , (0b0110111, RV32I_ALU.ALU_ADD), UInst),

  ('add'   , (0b0110011, 0b000, 0b0000000, RV32I_ALU.ALU_ADD), RInst),
  ('sub'   , (0b0110011, 0b000, 0b0100000, RV32I_ALU.ALU_SUB), RInst),

  ('jalr'  , (0b1100111, 0b000, RV32I_ALU.ALU_ADD, (RV32I_ALU.ALU_TRUE, False)), IInst),
  ('addi'  , (0b0010011, 0b000, RV32I_ALU.ALU_ADD, None), IInst),
  ('xori'  , (0b0010011, 0b100, RV32I_ALU.ALU_XOR, None), IInst),

  ('lw'    , (0b0000011, 0b010, RV32I_ALU.ALU_ADD, None), IInst),
  ('ebreak', (0b1110011, 0b000, None, None), IInst),

  # mn,       opcode,    funct3,cmp,                  flip
  ('bne'   , (0b1100011, 0b001, RV32I_ALU.ALU_CMP_EQ, True), BInst),
]

deocder_signals = Record(
  # prepare the operands
  rs1=Bits(5),
  rs1_valid=Bits(1),
  rs2=Bits(5),
  rs2_valid=Bits(1),
  rd=Bits(5),
  rd_valid=Bits(1),
  imm=Bits(32),
  imm_valid=Bits(1),
  # memory[0:0] is read, and memory[1:1] is write.
  memory=Bits(2),
  # bit vector of ALU operations, which result should be selected.
  alu=Bits(RV32I_ALU.CNT),
  # bit vector of conditions, which should be selected to write or branch.
  cond=Bits(RV32I_ALU.CNT),
  # if the result of condition should be flipped, because we only have < and ==.
  flip=Bits(1),
  # if the decoded instruction is a branch instruction.
  is_branch=Bits(1),
)

#TODO(@were): Add `SInst` to the supported types later.
supported_types = [RInst, IInst, BInst, UInst, JInst]
