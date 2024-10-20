
from assassyn.frontend import *
from opcodes import *
from instructions import *

def decode_logic(inst):

    views = {i: i(inst) for i in supported_types}
    is_type = {i: Bits(1)(0) for i in supported_types}

    eqs = {}

    rd_valid = Bits(1)(0)
    rs1_valid = Bits(1)(0)
    rs2_valid = Bits(1)(0)
    imm_valid = Bits(1)(0)
    supported = Bits(1)(0)

    alu = Bits(RV32I_ALU.CNT)(0)
    cond = Bits(RV32I_ALU.CNT)(0)
    flip = Bits(1)(0)

    log("raw: 0x{:08x}  |", inst)

    # Check if the given instruction's opcode equals one of the supported opcodes
    for mn, args, cur_type in supported_opcodes:

        ri = views[cur_type]
        signal = ri.decode(*args)
        eq = signal.eq
        is_type[cur_type] = is_type[cur_type] | eq
        eqs[mn] = eq
        supported = supported | eq

        # TODO(@were): Create a unified interface for these three signal gatherings
        alu = alu | eq.select(signal.alu, Bits(RV32I_ALU.CNT)(0))
        cond = cond | eq.select(signal.cond, Bits(RV32I_ALU.CNT)(0))
        flip = flip | eq.select(signal.flip, Bits(1)(0))

        pad = 6 - len(mn)
        pad = ' ' * pad

        fmt = None
        opcode = args[0]
        str_opcode = bin(opcode)[2:]
        str_opcode = (7 - len(str_opcode)) * '0' + str_opcode
        fmt = f"{cur_type.PREFIX}.{mn}.{str_opcode}{pad} "

        args = []

        if ri.has_rd:
            fmt = fmt + "| rd: x{:02}      "
            args.append(ri.view().rd)
            rd_valid = rd_valid | eq
        else:
            fmt = fmt + '|              '

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

    with Condition(~supported):
        view = views[RInst].view()
        log("Unsupported instruction: opcode = 0x{:x} funct3: 0x{:x} funct7: 0x{:x}", view.opcode, view.funct3, view.funct7)
        assume(Bits(1)(0))

    # Extract all the signals
    # For now, write is always disabled.
    memory = concat(Bits(1)(0), eqs['lw'])
    # BInst and JInst are designed for branches.
    is_branch = is_type[BInst] | is_type[JInst] | eqs['ebreak'] | eqs['jalr']
    # Extract all the operands according to the instruction types
    # rd
    rd = rd_valid.select(views[RInst].view().rd, Bits(5)(0))
    rs1 = rs1_valid.select(views[RInst].view().rs1, Bits(5)(0))
    rs2 = rs2_valid.select(views[RInst].view().rs2, Bits(5)(0))
    # imm
    # TODO(@were): Add `SInst` back to this list later.
    imm_valid = is_type[IInst] | is_type[UInst] | is_type[BInst]

    imm = Bits(32)(0)
    for i in supported_types:
        new_imm = views[i].imm(True)
        if new_imm is not None:
            imm = is_type[i].select(new_imm, imm)
    imm = eqs['lui'].select(views[UInst].imm(False).concat(Bits(12)(0)), imm)

    return deocder_signals.bundle(
        memory=memory,
        alu=alu,
        cond=cond,
        flip=flip,
        is_branch=is_branch,
        rs1=rs1,
        rs1_valid=rs1_valid,
        rs2=rs2,
        rs2_valid=rs2_valid,
        rd=rd,
        rd_valid=rd_valid,
        imm=imm,
        imm_valid=imm_valid)
