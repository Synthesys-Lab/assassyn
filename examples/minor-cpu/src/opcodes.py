from assassyn.frontend import Bits

class Opcode:
    LUI     = Bits(7)(0b0110111)
    ADDI    = Bits(7)(0b0010011)
    ADD     = Bits(7)(0b0110011)
    LW      = Bits(7)(0b0000011)
    BNE     = Bits(7)(0b1100011)
    RET     = Bits(7)(0b1101111)
    EBREAK  = Bits(7)(0b1110011)

class OpcodeChecker:
    def __init__(self, opcode):
        self.opcode = opcode
        self._results = {}

    def check(self, *types):
        for t in types:
            if t not in self._results:
                self._results[t] = (self.opcode == getattr(Opcode, t.upper()))
        return self

    def __getattr__(self, name):
        if name.upper() in dir(Opcode):
            return self.check(name)._results[name]
        raise AttributeError(f"'{self.__class__.__name__}' object has no attribute '{name}'")

def is_opcode(opcode, *types):
    return sum(opcode == getattr(Opcode, t.upper()) for t in types)

def is_lui(opcode):  return opcode == Opcode.LUI
def is_addi(opcode): return opcode == Opcode.ADDI
def is_add(opcode):  return opcode == Opcode.ADD
def is_lw(opcode):   return opcode == Opcode.LW
def is_bne(opcode):  return opcode == Opcode.BNE
def is_ret(opcode):  return opcode == Opcode.RET
def is_ebreak(opcode): return opcode == Opcode.EBREAK