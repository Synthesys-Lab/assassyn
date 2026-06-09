from assassyn.frontend import *
from opcodes import *

# The @rewrite_assign decorator captures variable names in methods to improve IR readability

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

    @rewrite_assign
    def decode(self, opcode, funct3, funct7, alu, ex_code=None):
        view = self.view()
        opcode = view.opcode == Bits(7)(opcode)
        funct3 = view.funct3 == Bits(3)(funct3)
        funct7 = view.funct7 == Bits(7)(funct7)
        if ex_code is not None:
            ex = view.rs2 == Bits(5)(ex_code)
        else:
            ex = Bits(1)(1)
        eq = opcode & funct3 & funct7 & ex
        return InstSignal(eq, alu)


    def imm(self, pad):
        return None

class IInst(InstType):

    PREFIX = 'i'

    def __init__(self, value):
        super().__init__(True, True, False, True, False, { (20, 31): ('imm', Bits) }, value)

    @rewrite_assign
    def imm(self, pad):
        raw = self.view().imm
        if pad:
            signal = raw[11:11]
            signal = signal.select(Bits(20)(0xfffff), Bits(20)(0))
            raw = concat(signal, raw)
        return raw

    @rewrite_assign
    def decode(self, opcode, funct3, alu, cond , ex_code ,ex_code2):
        view = self.view()
        opcode = view.opcode == Bits(7)(opcode)
        funct3 = view.funct3 == Bits(3)(funct3)
        if ex_code is not None:
            ex = view.imm == Bits(12)(ex_code)
        else:
            ex = Bits(1)(1)
        
        if ex_code2 is not None:
            ex2 = (view.imm)[6:11] == Bits(6)(ex_code2)
            #log("ex2_code: 0x{:x} | imm[6:11]: 0x{:x}", Bits(6)(ex_code2) , (view.imm)[6:11])
        else:
            ex2 = Bits(1)(1)
        
        eq = opcode & funct3 & ex & ex2
        return InstSignal(eq, alu, cond=cond)

class SInst(InstType):

    PREFIX = 's'

    def __init__(self, value):
        fields = { (25, 31): ('imm11_5', Bits), (7, 11): ('imm4_0', Bits) }
        super().__init__(False, True, True, True, False, fields, value)

    @rewrite_assign
    def decode(self, opcode, funct3, alu):
        view = self.view()
        opcode = view.opcode == Bits(7)(opcode)
        funct3 = view.funct3 == Bits(3)(funct3)
        eq = opcode & funct3
        return InstSignal(eq, alu)

    @rewrite_assign
    def imm(self, pad):
        imm = self.view().imm11_5.concat(self.view().imm4_0)
        if pad:
            msb = imm[11:11]
            msb = msb.select(Bits(20)(0xfffff), Bits(20)(0))
            imm = concat(msb, imm)
        return imm

class UInst(InstType):

    PREFIX = 'u'

    def __init__(self, value):
        super().__init__(True, False, False, False, False, { (12, 31): ('imm', Bits) }, value)

    @rewrite_assign
    def decode(self, opcode, alu):
        view = self.view()
        eq = view.opcode == Bits(7)(opcode)
        return InstSignal(eq, alu)

    @rewrite_assign
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

    @rewrite_assign
    def decode(self, opcode, alu, cond):
        view = self.view()
        eq = view.opcode == Bits(7)(opcode)
        return InstSignal(eq, alu, cond=cond)

    @rewrite_assign
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

    @rewrite_assign
    def decode(self, opcode, funct3, cmp, flip):
        view = self.view()
        opcode = view.opcode == Bits(7)(opcode)
        funct3 = view.funct3 == Bits(3)(funct3)
        eq = opcode & funct3
        return InstSignal(eq, RV32I_ALU.ALU_ADD, cond=(cmp, flip))

    @rewrite_assign
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
    ALU_ORI = 13
    ALU_AND = 4
    ALU_SLL = 5
    ALU_SRL = 6
    ALU_SRA = 7
    ALU_SRA_U = 12
    ALU_CMP_EQ = 8
    ALU_CMP_LT = 9
    ALU_CMP_LTU = 10
    # Always true.
    ALU_TRUE = 11
    ALU_NONE = 15

supported_opcodes = [
  # mn,     opcode,      operator
  ('jal'   , (0b1101111, RV32I_ALU.ALU_ADD , (RV32I_ALU.ALU_TRUE, False)), JInst),

  ('lui'   , (0b0110111, RV32I_ALU.ALU_ADD), UInst),

  ('add'   , (0b0110011, 0b000, 0b0000000, RV32I_ALU.ALU_ADD), RInst),
  ('sub'   , (0b0110011, 0b000, 0b0100000, RV32I_ALU.ALU_SUB), RInst),
  ('or'    , (0b0110011, 0b110, 0b0000000, RV32I_ALU.ALU_OR) , RInst),

  ('jalr'  , (0b1100111, 0b000, RV32I_ALU.ALU_ADD, (RV32I_ALU.ALU_TRUE, False), None, None), IInst),
  ('addi'  , (0b0010011, 0b000, RV32I_ALU.ALU_ADD, None, None, None), IInst),


  ('lw'    , (0b0000011, 0b010, RV32I_ALU.ALU_ADD, None, None, None), IInst),
  ('lbu'   , (0b0000011, 0b100, RV32I_ALU.ALU_ADD, None, None, None), IInst),

  ('ebreak', (0b1110011, 0b000, RV32I_ALU.ALU_NONE, None,0b000000000001,None), IInst),

  ('sw'    , (0b0100011, 0b010, RV32I_ALU.ALU_ADD), SInst),

  # mn,       opcode,    funct3,cmp,                  flip
  ('beq'   , (0b1100011, 0b000, RV32I_ALU.ALU_CMP_EQ,  False), BInst),
  ('bne'   , (0b1100011, 0b001, RV32I_ALU.ALU_CMP_EQ,  True), BInst),
  ('blt'   , (0b1100011, 0b100, RV32I_ALU.ALU_CMP_LT,  False), BInst),
  ('bge'   , (0b1100011, 0b101, RV32I_ALU.ALU_CMP_LT,  True), BInst),
  ('bgeu' , (0b1100011, 0b111, RV32I_ALU.ALU_CMP_LTU, True), BInst),
  ('bltu' , (0b1100011, 0b110, RV32I_ALU.ALU_CMP_LTU, False), BInst),

  ('csrrs'   , (0b1110011, 0b010, RV32I_ALU.ALU_OR, None ,None ,None), IInst),
  ('auipc' , (0b0010111, RV32I_ALU.ALU_ADD), UInst),
  ('csrrw' , (0b1110011, 0b001, RV32I_ALU.ALU_ADD, None,None,None), IInst),
  ('csrrwi' , (0b1110011, 0b101, RV32I_ALU.ALU_ADD, None,None,None), IInst),

  ('slli' , (0b0010011, 0b001, RV32I_ALU.ALU_SLL, None, None , 0b000000), IInst),
  ('sll'  , (0b0110011, 0b001, 0b0000000, RV32I_ALU.ALU_SLL), RInst),
  ('srai' , (0b0010011, 0b101, RV32I_ALU.ALU_SRA,  None,None , 0b010000), IInst),#signed
  ('srli' , (0b0010011, 0b101, RV32I_ALU.ALU_SRA_U,  None, None , 0b000000), IInst),#0
  ('sltu' , (0b0110011, 0b011, 0b0000000, RV32I_ALU.ALU_CMP_LTU), RInst),
  ('srl'  , (0b0110011, 0b101, 0b0000000, RV32I_ALU.ALU_SRA_U), RInst),
  ('sra'  , (0b0110011, 0b101, 0b0100000, RV32I_ALU.ALU_SRA), RInst),

  #todo: mret is not supported for setting the MPIE in CSR(mstatus)
  ('mret' , (0b1110011, 0b000, 0b0011000,RV32I_ALU.ALU_ADD,0b00010), RInst ),
  #we have only a sigle thread, so we don't need to deal with 'fence' instruction
  ('fence' , (0b0001111, 0b000, RV32I_ALU.ALU_ADD, None,None,None), IInst),
  ('ecall' , (0b1110011, 0b000, RV32I_ALU.ALU_NONE, None,0b000000000000,None), IInst),
  
  ('and' , (0b0110011, 0b111, 0b0000000, RV32I_ALU.ALU_AND), RInst),
  ('andi' , (0b0010011, 0b111, RV32I_ALU.ALU_AND, None,None,None), IInst),
  ('ori' , (0b0010011, 0b110, RV32I_ALU.ALU_ORI, None,None,None), IInst),
  ('xori' , (0b0010011, 0b100, RV32I_ALU.ALU_XOR, None,None,None), IInst),
]

deocder_signals = Record(
  # prepare the operands
  rs1=Bits(5),
  rs1_valid=Bits(1),
  rs2=Bits(5),
  rs2_valid=Bits(1),
  rd=Bits(5),
  rd_valid=Bits(1),
  csr_read = Bits(1),
  csr_write = Bits(1),
  csr_calculate = Bits(1),
  is_zimm = Bits(1),
  is_mepc = Bits(1), 
  is_pc_calc = Bits(1),
  #csr_id = Bits(4),
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
  # b-inst and jal are both using the instruction's pc as the base of branching,
  # while jalr is using the rs1 as the base of branching.
  is_offset_br=Bits(1),
  # should we link the pc to rd
  link_pc=Bits(1),
  mem_ext=Bits(2),
)

#TODO(@were): Add `SInst` to the supported types later.
supported_types = [RInst, IInst, BInst, UInst, JInst, SInst]
