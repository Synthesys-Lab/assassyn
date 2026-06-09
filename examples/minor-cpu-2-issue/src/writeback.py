from assassyn.frontend import *

EPOCH_BITS = 2
EPOCH_DTYPE = Bits(EPOCH_BITS)


def commit_writeback(reg_file, rd, mdata, log_label):
    with Condition((rd != Bits(5)(0))):
        log(f"{log_label:16} | x{{:02}}          | 0x{{:08x}}", rd, mdata)
        reg_file[rd] = mdata


class WriteBack0(Module):
    def __init__(self):
        super().__init__(
            ports={
                "rd": Port(Bits(5)),
                "mdata": Port(Bits(32)),
                "epoch": Port(EPOCH_DTYPE),
            },
            no_arbiter=True,
        )
        self.name = "W0"

    @module.combinational
    def build(
        self,
        reg_file: Array,
        wb0_bypass_reg: Array,
        wb0_bypass_data: Array,
        wb0_bypass_epoch: Array,
    ):
        rd, mdata, epoch = self.pop_all_ports(False)
        wb0_bypass_reg[0] = rd
        wb0_bypass_data[0] = mdata
        wb0_bypass_epoch[0] = epoch
        commit_writeback(reg_file, rd, mdata, "writeback0")
        return rd


class WriteBack1(Module):
    def __init__(self):
        super().__init__(
            ports={
                "rd": Port(Bits(5)),
                "mdata": Port(Bits(32)),
                "epoch": Port(EPOCH_DTYPE),
            },
            no_arbiter=True,
        )
        self.name = "W1"

    @module.combinational
    def build(
        self,
        reg_file: Array,
        wb1_bypass_reg: Array,
        wb1_bypass_data: Array,
        wb1_bypass_epoch: Array,
    ):
        rd, mdata, epoch = self.pop_all_ports(False)
        wb1_bypass_reg[0] = rd
        wb1_bypass_data[0] = mdata
        wb1_bypass_epoch[0] = epoch
        commit_writeback(reg_file, rd, mdata, "writeback1")
        return rd
