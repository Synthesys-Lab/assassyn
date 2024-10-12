
from assassyn.frontend import *
from opcodes import *
from instruction_types import *

def decode_logic(inst):

    views = {i: i(inst) for i in supported_types}
    is_type = {i: Bits(1)(0) for i in supported_types}

    eqs = {}

    rd_valid = Bits(1)(0)
    rs1_valid = Bits(1)(0)
    rs2_valid = Bits(1)(0)
    imm_valid = Bits(1)(0)

    # Check if the given instruction's opcode equals one of the supported opcodes
    for mn, opcode, cur_type in supported_opcodes:

        ri = views[cur_type]
        wrapped_opcode = Bits(7)(opcode)
        eq = ri.view().opcode == wrapped_opcode
        is_type[cur_type] = is_type[cur_type] | eq
        eqs[mn] = eq

        pad = 6 - len(mn)
        pad = ' ' * pad


        fmt = None
        str_opcode = bin(opcode)[2:]
        str_opcode = (7 - len(str_opcode)) * '0' + str_opcode
        fmt = f"{cur_type.PREFIX}.{mn}.{str_opcode}{pad} "

        args = []

        if ri.has_rd:
            fmt = fmt + "| rd: x{:02}      "
            args.append(ri.view().rd)
            rd_valid = rd_valid | eq
        else:
            fmt = fmt + '|               '

        if ri.has_rs1:
            fmt = fmt + "| rs1: x{:02}      "
            args.append(ri.view().rs1)
            rs1_valid = rs1_valid | eq
        else:
            fmt = fmt + '|               '

        if ri.has_rs2:
            fmt = fmt + "| rs2: x{:02}      "
            args.append(ri.view().rs2)
            rs2_valid = rs2_valid | eq
        else:
            fmt = fmt + '|               '

        imm = ri.imm(False)
        if imm is not None:
            fmt = fmt + "|imm: 0x{:x}"
            args.append(imm)

        with Condition(eq):
            log(fmt, *args)

    # Extract all the signals
    memory_read = eqs['lw']
    invoke_adder = eqs['addi'] | eqs['add'] | eqs['lw'] | eqs['bne']
    is_branch = eqs['bne'] | eqs['ret'] | eqs['ebreak']
    # Extract all the operands according to the instruction types
    # rd
    rd_reg = rd_valid.select(views[RInst].view().rd, Bits(5)(0))
    # rs1
    rs1_reg = rs1_valid.select(views[RInst].view().rs1, Bits(5)(0))
    # rs2
    rs2_reg = rs2_valid.select(views[RInst].view().rs2, Bits(5)(0))
    # imm
    imm_valid = is_type[IInst] | is_type[UInst] | is_type[SInst] | is_type[BInst]
    imm_value = Bits(32)(0)
    for i in supported_types:
        new_imm = views[i].imm(True)
        if new_imm is not None:
            imm_value = is_type[i].select(new_imm, imm_value)
    imm_value = eqs['lui'].select(views[UInst].imm(False).concat(Bits(12)(0)), imm_value)

    return deocder_signals.bundle(
        memory_read=memory_read,
        invoke_adder=invoke_adder,
        is_branch=is_branch,
        rs1_reg=rs1_reg,
        rs1_valid=rs1_valid,
        rs2_reg=rs2_reg,
        rs2_valid=rs2_valid,
        rd_reg=rd_reg,
        rd_valid=rd_valid,
        imm_valid=imm_valid,
        imm_value=imm_value)
