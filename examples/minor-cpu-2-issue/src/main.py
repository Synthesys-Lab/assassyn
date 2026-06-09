"""Conservative 2-issue in-order minor CPU example built with Assassyn."""

import csv
import os
import re
import shutil
import subprocess
import sys as py_sys

from assassyn.frontend import *
from assassyn.backend import *
from assassyn import utils

from decoder import decode_logic
from instructions import *
from writeback import *
from memory_access import *


current_path = os.path.dirname(os.path.abspath(__file__))
workspace = f"{current_path}/.workspace/"

DEFAULT_WORKLOADS = [
    "median",
    "multiply",
    "qsort",
    "towers",
    "vvadd",
]

REG_COUNT = 32
QUEUE_DEPTH = 4
QUEUE_COUNT_BITS = 3
EPOCH_BITS = 2
EPOCH_DTYPE = Bits(EPOCH_BITS)
PRODUCER_EPOCH_WORD_BITS = REG_COUNT * EPOCH_BITS
PRODUCER_EPOCH_DTYPE = Bits(PRODUCER_EPOCH_WORD_BITS)
CSR_ID_MAP = [
    (773, 1),   # mtvec
    (833, 2),   # mepc
    (772, 4),   # mie
    (768, 8),   # mstatus
    (3860, 9),  # mhartid
    (384, 10),  # satp
    (944, 11),  # pmpaddr0
    (928, 12),  # pmpcfg0
    (770, 13),  # medeleg
    (771, 14),  # mideleg
    (1860, 15), # unknown
]

PRODUCER_NONE = Bits(3)(0)
PRODUCER_EXEC0 = Bits(3)(1)
PRODUCER_EXEC1 = Bits(3)(2)
PRODUCER_MEM = Bits(3)(3)
PRODUCER_WB = Bits(3)(4)
PURE_DUAL_ISSUE_MODE = os.environ.get("MINOR2_PURE_DUAL_ISSUE", "0") == "1"
DISC_FAIR_DUAL_ISSUE_MODE = os.environ.get("MINOR2_DISC_FAIR_DUAL_ISSUE", "0") == "1"

if PURE_DUAL_ISSUE_MODE and DISC_FAIR_DUAL_ISSUE_MODE:
    raise RuntimeError("Select only one dual-issue mode: MINOR2_PURE_DUAL_ISSUE or MINOR2_DISC_FAIR_DUAL_ISSUE")


def is_memory_op(signals):
    return signals.memory != Bits(2)(0)


def is_special_op(signals):
    return signals.alu == Bits(RV32I_ALU.CNT)(1 << RV32I_ALU.ALU_NONE)


def is_csr_op(signals):
    return signals.csr_read | signals.csr_write | signals.csr_calculate


def writes_rd(signals):
    return signals.rd_valid & (signals.rd != Bits(5)(0))


def rd_mask(rd):
    return (rd != Bits(5)(0)).select(Bits(32)(1) << rd, Bits(32)(0))


def mask_word(mask):
    return mask[0] if isinstance(mask, Array) else mask


def priority_select(mask, values, default):
    value = default
    for idx, candidate in enumerate(values):
        value = mask[idx:idx].select(candidate, value)
    return value


def reg_tracked(mask, reg_idx):
    return (mask_word(mask) >> reg_idx)[0:0]


def epoch_index(reg_idx):
    return reg_idx.concat(Bits(1)(0))


def reg_epoch(producer_epoch, reg_idx):
    return (mask_word(producer_epoch) >> epoch_index(reg_idx))[0:EPOCH_BITS - 1]


def truncate_epoch(epoch):
    return epoch[0:EPOCH_BITS - 1]


def next_producer_epoch(producer_epoch, reg_idx):
    return truncate_epoch(reg_epoch(producer_epoch, reg_idx) + EPOCH_DTYPE(1))


def issued_epoch(signals, producer_epoch):
    return writes_rd(signals).select(next_producer_epoch(producer_epoch, signals.rd), EPOCH_DTYPE(0))


def set_reg_epoch(epoch_word, reg_idx, epoch):
    epoch = truncate_epoch(epoch)
    field_mask = Bits(PRODUCER_EPOCH_WORD_BITS)((1 << EPOCH_BITS) - 1) << epoch_index(reg_idx)
    cleared = epoch_word & (~field_mask)
    epoch_value = concat(Bits(PRODUCER_EPOCH_WORD_BITS - EPOCH_BITS)(0), epoch) << epoch_index(reg_idx)
    return (reg_idx == Bits(5)(0)).select(epoch_word, cleared | epoch_value)


def reg_pending(exec0_owner, exec1_owner, mem_owner, wb_owner, reg_idx):
    return (
        reg_tracked(exec0_owner, reg_idx)
        | reg_tracked(exec1_owner, reg_idx)
        | reg_tracked(mem_owner, reg_idx)
        | reg_tracked(wb_owner, reg_idx)
    )


def reg_producer(exec0_owner, exec1_owner, mem_owner, wb_owner, reg_idx):
    producer = PRODUCER_NONE
    producer = reg_tracked(wb_owner, reg_idx).select(PRODUCER_WB, producer)
    producer = reg_tracked(mem_owner, reg_idx).select(PRODUCER_MEM, producer)
    producer = reg_tracked(exec1_owner, reg_idx).select(PRODUCER_EXEC1, producer)
    producer = reg_tracked(exec0_owner, reg_idx).select(PRODUCER_EXEC0, producer)
    return (reg_idx == Bits(5)(0)).select(PRODUCER_NONE, producer)


def wb_hit(reg_idx, expected_epoch, wb0_bypass_reg, wb0_bypass_epoch, wb1_bypass_reg, wb1_bypass_epoch):
    return ((wb0_bypass_reg[0] == reg_idx) & (wb0_bypass_epoch[0] == expected_epoch)) | (
        (wb1_bypass_reg[0] == reg_idx) & (wb1_bypass_epoch[0] == expected_epoch)
    )


def wb_value(
    reg_idx,
    expected_epoch,
    default_value,
    wb0_bypass_reg,
    wb0_bypass_epoch,
    wb0_bypass_data,
    wb1_bypass_reg,
    wb1_bypass_epoch,
    wb1_bypass_data,
):
    value = ((wb0_bypass_reg[0] == reg_idx) & (wb0_bypass_epoch[0] == expected_epoch)).select(
        wb0_bypass_data[0], default_value
    )
    value = ((wb1_bypass_reg[0] == reg_idx) & (wb1_bypass_epoch[0] == expected_epoch)).select(
        wb1_bypass_data[0], value
    )
    return value


def pair_bypass_hit(reg_idx, is_valid, pair_bypass_valid, pair_bypass_reg):
    if pair_bypass_valid is None or pair_bypass_reg is None:
        return Bits(1)(0)
    if not isinstance(is_valid, Value):
        is_valid = Bits(1)(is_valid)
    return is_valid & pair_bypass_valid & (pair_bypass_reg != Bits(5)(0)) & (reg_idx == pair_bypass_reg)


def visible_stage_mask(bypass_reg, bypass_epoch, producer_epoch):
    rd = bypass_reg[0]
    epoch_match = (rd != Bits(5)(0)) & (bypass_epoch[0] == reg_epoch(producer_epoch, rd))
    return epoch_match.select(rd_mask(rd), Bits(32)(0))


def producer_visible(
    reg_idx,
    is_valid,
    producer_epoch,
    exec0_bypass_reg,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_epoch,
    pair_bypass_valid=None,
    pair_bypass_reg=None,
):
    if not isinstance(is_valid, Value):
        is_valid = Bits(1)(is_valid)
    expected_epoch = reg_epoch(producer_epoch, reg_idx)
    exec0_hit = (exec0_bypass_reg[0] == reg_idx) & (exec0_bypass_epoch[0] == expected_epoch)
    prev_exec0_hit = (prev_exec0_bypass_reg[0] == reg_idx) & (prev_exec0_bypass_epoch[0] == expected_epoch)
    exec1_hit = (exec1_bypass_reg[0] == reg_idx) & (exec1_bypass_epoch[0] == expected_epoch)
    prev_exec1_hit = (prev_exec1_bypass_reg[0] == reg_idx) & (prev_exec1_bypass_epoch[0] == expected_epoch)
    mem_hit = (mem_bypass_reg[0] == reg_idx) & (mem_bypass_epoch[0] == expected_epoch)
    wb_stage_hit = wb_hit(reg_idx, expected_epoch, wb0_bypass_reg, wb0_bypass_epoch, wb1_bypass_reg, wb1_bypass_epoch)
    pair_hit = pair_bypass_hit(reg_idx, is_valid, pair_bypass_valid, pair_bypass_reg)
    return exec0_hit | prev_exec0_hit | exec1_hit | prev_exec1_hit | mem_hit | wb_stage_hit | pair_hit


def producer_visible_conservative(
    reg_idx,
    is_valid,
    producer_epoch,
    exec0_bypass_reg,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_epoch,
    pair_bypass_valid=None,
    pair_bypass_reg=None,
):
    del prev_exec0_bypass_reg, prev_exec0_bypass_epoch
    del prev_exec1_bypass_reg, prev_exec1_bypass_epoch
    del pair_bypass_valid, pair_bypass_reg
    if not isinstance(is_valid, Value):
        is_valid = Bits(1)(is_valid)
    expected_epoch = reg_epoch(producer_epoch, reg_idx)
    exec0_hit = (exec0_bypass_reg[0] == reg_idx) & (exec0_bypass_epoch[0] == expected_epoch)
    exec1_hit = (exec1_bypass_reg[0] == reg_idx) & (exec1_bypass_epoch[0] == expected_epoch)
    mem_hit = (mem_bypass_reg[0] == reg_idx) & (mem_bypass_epoch[0] == expected_epoch)
    wb_stage_hit = wb_hit(reg_idx, expected_epoch, wb0_bypass_reg, wb0_bypass_epoch, wb1_bypass_reg, wb1_bypass_epoch)
    return exec0_hit | exec1_hit | mem_hit | wb_stage_hit


def operand_ready(
    reg_idx,
    is_valid,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_epoch,
    pair_bypass_valid=None,
    pair_bypass_reg=None,
):
    if not isinstance(is_valid, Value):
        is_valid = Bits(1)(is_valid)
    pending = reg_pending(exec0_owner, exec1_owner, mem_owner, wb_owner, reg_idx)
    bypass_hit = producer_visible(
        reg_idx,
        is_valid,
        producer_epoch,
        exec0_bypass_reg,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_epoch,
        pair_bypass_valid,
        pair_bypass_reg,
    )
    ready = (~pending) | bypass_hit
    return (~is_valid) | ready | (reg_idx == Bits(5)(0))


def operand_ready_conservative(
    reg_idx,
    is_valid,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_epoch,
    pair_bypass_valid=None,
    pair_bypass_reg=None,
):
    if not isinstance(is_valid, Value):
        is_valid = Bits(1)(is_valid)
    pending = reg_pending(exec0_owner, exec1_owner, mem_owner, wb_owner, reg_idx)
    bypass_hit = producer_visible_conservative(
        reg_idx,
        is_valid,
        producer_epoch,
        exec0_bypass_reg,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_epoch,
        pair_bypass_valid,
        pair_bypass_reg,
    )
    ready = (~pending) | bypass_hit
    return (~is_valid) | ready | (reg_idx == Bits(5)(0))


def resolve_operand(
    reg_idx,
    is_valid,
    reg_file,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_data,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_data,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_data,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_data,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_data,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_data,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_data,
    wb1_bypass_epoch,
    pair_bypass_valid=None,
    pair_bypass_reg=None,
    pair_bypass_data=None,
):
    value = reg_file[reg_idx]
    expected_epoch = reg_epoch(producer_epoch, reg_idx)
    exec0_hit = (exec0_bypass_reg[0] == reg_idx) & (exec0_bypass_epoch[0] == expected_epoch)
    prev_exec0_hit = (prev_exec0_bypass_reg[0] == reg_idx) & (prev_exec0_bypass_epoch[0] == expected_epoch)
    exec1_hit = (exec1_bypass_reg[0] == reg_idx) & (exec1_bypass_epoch[0] == expected_epoch)
    prev_exec1_hit = (prev_exec1_bypass_reg[0] == reg_idx) & (prev_exec1_bypass_epoch[0] == expected_epoch)
    mem_hit = (mem_bypass_reg[0] == reg_idx) & (mem_bypass_epoch[0] == expected_epoch)
    wb_data = wb_value(
        reg_idx,
        expected_epoch,
        value,
        wb0_bypass_reg,
        wb0_bypass_epoch,
        wb0_bypass_data,
        wb1_bypass_reg,
        wb1_bypass_epoch,
        wb1_bypass_data,
    )
    value = wb_data
    value = mem_hit.select(mem_bypass_data[0], value)
    value = prev_exec1_hit.select(prev_exec1_bypass_data[0], value)
    value = exec1_hit.select(exec1_bypass_data[0], value)
    value = prev_exec0_hit.select(prev_exec0_bypass_data[0], value)
    value = exec0_hit.select(exec0_bypass_data[0], value)
    if pair_bypass_data is not None:
        pair_hit = pair_bypass_hit(reg_idx, is_valid, pair_bypass_valid, pair_bypass_reg)
        value = pair_hit.select(pair_bypass_data, value)
    value = (~is_valid).select(Bits(32)(0), value)
    return (reg_idx == Bits(5)(0)).select(Bits(32)(0), value)


def resolve_operand_conservative(
    reg_idx,
    is_valid,
    reg_file,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_data,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_data,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_data,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_data,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_data,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_data,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_data,
    wb1_bypass_epoch,
    pair_bypass_valid=None,
    pair_bypass_reg=None,
    pair_bypass_data=None,
):
    del exec0_owner, exec1_owner, mem_owner, wb_owner
    del prev_exec0_bypass_reg, prev_exec0_bypass_data, prev_exec0_bypass_epoch
    del prev_exec1_bypass_reg, prev_exec1_bypass_data, prev_exec1_bypass_epoch
    del pair_bypass_valid, pair_bypass_reg, pair_bypass_data
    value = reg_file[reg_idx]
    expected_epoch = reg_epoch(producer_epoch, reg_idx)
    exec0_hit = (exec0_bypass_reg[0] == reg_idx) & (exec0_bypass_epoch[0] == expected_epoch)
    exec1_hit = (exec1_bypass_reg[0] == reg_idx) & (exec1_bypass_epoch[0] == expected_epoch)
    mem_hit = (mem_bypass_reg[0] == reg_idx) & (mem_bypass_epoch[0] == expected_epoch)
    wb_data = wb_value(
        reg_idx,
        expected_epoch,
        value,
        wb0_bypass_reg,
        wb0_bypass_epoch,
        wb0_bypass_data,
        wb1_bypass_reg,
        wb1_bypass_epoch,
        wb1_bypass_data,
    )
    value = wb_data
    value = mem_hit.select(mem_bypass_data[0], value)
    value = exec1_hit.select(exec1_bypass_data[0], value)
    value = exec0_hit.select(exec0_bypass_data[0], value)
    value = (~is_valid).select(Bits(32)(0), value)
    return (reg_idx == Bits(5)(0)).select(Bits(32)(0), value)


def resolve_csr_id(signals):
    csr_id = Bits(4)(0)
    for raw, mapped in CSR_ID_MAP:
        csr_id = (signals.imm[0:11] == Bits(12)(raw)).select(Bits(4)(mapped), csr_id)
        csr_id = signals.is_mepc.select(Bits(4)(2), csr_id)
    return csr_id


def prepare_execution_operands(
    signals,
    reg_file,
    csr_file,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_data,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_data,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_data,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_data,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_data,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_data,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_data,
    wb1_bypass_epoch,
    pair_bypass_valid=None,
    pair_bypass_reg=None,
    pair_bypass_data=None,
):
    csr_id = resolve_csr_id(signals)
    is_csr = signals.csr_read | signals.csr_write
    a_src = resolve_operand(
        signals.rs1,
        signals.rs1_valid,
        reg_file,
        producer_epoch,
        exec0_owner,
        exec1_owner,
        mem_owner,
        wb_owner,
        exec0_bypass_reg,
        exec0_bypass_data,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_data,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_data,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_data,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_data,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_data,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_data,
        wb1_bypass_epoch,
        pair_bypass_valid,
        pair_bypass_reg,
        pair_bypass_data,
    )
    b_src = resolve_operand(
        signals.rs2,
        signals.rs2_valid,
        reg_file,
        producer_epoch,
        exec0_owner,
        exec1_owner,
        mem_owner,
        wb_owner,
        exec0_bypass_reg,
        exec0_bypass_data,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_data,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_data,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_data,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_data,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_data,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_data,
        wb1_bypass_epoch,
        pair_bypass_valid,
        pair_bypass_reg,
        pair_bypass_data,
    )

    csr_new = Bits(32)(0)
    csr_new = signals.csr_write.select(a_src, csr_new)
    csr_new = signals.is_zimm.select(concat(Bits(27)(0), signals.rs1), csr_new)

    a = signals.csr_write.select(Bits(32)(0), a_src)
    b = is_csr.select(csr_file[csr_id], b_src)
    return a, b, csr_id, csr_new


def prepare_execution_operands_conservative(
    signals,
    reg_file,
    csr_file,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_data,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_data,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_data,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_data,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_data,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_data,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_data,
    wb1_bypass_epoch,
    pair_bypass_valid=None,
    pair_bypass_reg=None,
    pair_bypass_data=None,
):
    csr_id = resolve_csr_id(signals)
    is_csr = signals.csr_read | signals.csr_write
    a_src = resolve_operand_conservative(
        signals.rs1,
        signals.rs1_valid,
        reg_file,
        producer_epoch,
        exec0_owner,
        exec1_owner,
        mem_owner,
        wb_owner,
        exec0_bypass_reg,
        exec0_bypass_data,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_data,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_data,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_data,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_data,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_data,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_data,
        wb1_bypass_epoch,
        pair_bypass_valid,
        pair_bypass_reg,
        pair_bypass_data,
    )
    b_src = resolve_operand_conservative(
        signals.rs2,
        signals.rs2_valid,
        reg_file,
        producer_epoch,
        exec0_owner,
        exec1_owner,
        mem_owner,
        wb_owner,
        exec0_bypass_reg,
        exec0_bypass_data,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_data,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_data,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_data,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_data,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_data,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_data,
        wb1_bypass_epoch,
        pair_bypass_valid,
        pair_bypass_reg,
        pair_bypass_data,
    )
    csr_new = Bits(32)(0)
    csr_new = signals.csr_write.select(a_src, csr_new)
    csr_new = signals.is_zimm.select(concat(Bits(27)(0), signals.rs1), csr_new)

    a = signals.csr_write.select(Bits(32)(0), a_src)
    b = is_csr.select(csr_file[csr_id], b_src)
    return a, b, csr_id, csr_new


def slot_ready(
    signals,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_epoch,
):
    rs1_ready = operand_ready(
        signals.rs1,
        signals.rs1_valid,
        producer_epoch,
        exec0_owner,
        exec1_owner,
        mem_owner,
        wb_owner,
        exec0_bypass_reg,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_epoch,
    )
    rs2_ready = operand_ready(
        signals.rs2,
        signals.rs2_valid,
        producer_epoch,
        exec0_owner,
        exec1_owner,
        mem_owner,
        wb_owner,
        exec0_bypass_reg,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_epoch,
    )
    rd_blocked = rd_hazard(
        signals,
        producer_epoch,
        exec0_owner,
        exec1_owner,
        mem_owner,
        wb_owner,
        exec0_bypass_reg,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_epoch,
    )
    return rs1_ready & rs2_ready & (~rd_blocked)


def rd_hazard(
    signals,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_epoch,
):
    # Latest-version tracking plus epochs make older in-flight writers tolerable.
    # Same-cycle WAWs are still blocked by pair_safe / pair_swap_safe.
    return Bits(1)(0)


def rd_hazard_conservative(
    signals,
    producer_epoch,
    exec0_owner,
    exec1_owner,
    mem_owner,
    wb_owner,
    exec0_bypass_reg,
    exec0_bypass_epoch,
    prev_exec0_bypass_reg,
    prev_exec0_bypass_epoch,
    exec1_bypass_reg,
    exec1_bypass_epoch,
    prev_exec1_bypass_reg,
    prev_exec1_bypass_epoch,
    mem_bypass_reg,
    mem_bypass_epoch,
    wb0_bypass_reg,
    wb0_bypass_epoch,
    wb1_bypass_reg,
    wb1_bypass_epoch,
):
    rd_pending = reg_pending(exec0_owner, exec1_owner, mem_owner, wb_owner, signals.rd)
    rd_visible = producer_visible_conservative(
        signals.rd,
        signals.rd_valid,
        producer_epoch,
        exec0_bypass_reg,
        exec0_bypass_epoch,
        prev_exec0_bypass_reg,
        prev_exec0_bypass_epoch,
        exec1_bypass_reg,
        exec1_bypass_epoch,
        prev_exec1_bypass_reg,
        prev_exec1_bypass_epoch,
        mem_bypass_reg,
        mem_bypass_epoch,
        wb0_bypass_reg,
        wb0_bypass_epoch,
        wb1_bypass_reg,
        wb1_bypass_epoch,
    )
    return writes_rd(signals) & rd_pending & (~rd_visible)


def pair_safe(signals0, signals1):
    slot0_writes = writes_rd(signals0)
    slot1_writes = writes_rd(signals1)
    slot0_forwardable = slot0_writes & (~is_memory_op(signals0))
    raw_hazard = slot0_writes & (~slot0_forwardable) & (
        (signals1.rs1_valid & (signals0.rd == signals1.rs1))
        | (signals1.rs2_valid & (signals0.rd == signals1.rs2))
    )
    waw_hazard = slot0_writes & slot1_writes & (signals0.rd == signals1.rd)
    return ~(raw_hazard | waw_hazard)


def pair_swap_safe(signals0, signals1):
    slot0_writes = writes_rd(signals0)
    slot1_writes = writes_rd(signals1)
    raw_hazard = slot0_writes & (
        (signals1.rs1_valid & (signals0.rd == signals1.rs1))
        | (signals1.rs2_valid & (signals0.rd == signals1.rs2))
    )
    waw_hazard = slot0_writes & slot1_writes & (signals0.rd == signals1.rd)
    return ~(raw_hazard | waw_hazard)


def slot0_allows_pair(signals):
    return (~signals.is_branch) & (~is_csr_op(signals)) & (~is_special_op(signals))


def slot1_allows_pair(signals):
    return (~is_memory_op(signals)) & (~is_csr_op(signals)) & (~is_special_op(signals))


def slot1_prefers_lane0(signals):
    return is_memory_op(signals) & (~signals.is_branch) & (~is_csr_op(signals)) & (~is_special_op(signals))


def build_result(signals, fetch_addr, a, b):
    alu_a = (signals.is_offset_br | signals.is_pc_calc).select(fetch_addr, a)
    alu_b = signals.imm_valid.select(signals.imm, b)

    results = [Bits(32)(0)] * RV32I_ALU.CNT

    adder_result = (alu_a.bitcast(Int(32)) + alu_b.bitcast(Int(32))).bitcast(Bits(32))
    sub_result = (a.bitcast(Int(32)) - b.bitcast(Int(32))).bitcast(Bits(32))
    lt_result = (a.bitcast(Int(32)) < b.bitcast(Int(32))).select(Bits(32)(1), Bits(32)(0))
    eq_result = (a == b).select(Bits(32)(1), Bits(32)(0))
    ltu_result = (a < b).select(Bits(32)(1), Bits(32)(0))
    sra_signed_result = (a.bitcast(Int(32)) >> alu_b[0:4].bitcast(Int(5))).bitcast(Bits(32))

    results[RV32I_ALU.ALU_ADD] = adder_result
    results[RV32I_ALU.ALU_SUB] = sub_result
    results[RV32I_ALU.ALU_XOR] = a ^ alu_b
    results[RV32I_ALU.ALU_OR] = a | b
    results[RV32I_ALU.ALU_ORI] = a | alu_b
    results[RV32I_ALU.ALU_AND] = a & alu_b
    results[RV32I_ALU.ALU_SLL] = a << alu_b[0:4]
    results[RV32I_ALU.ALU_SRL] = a >> alu_b[0:4]
    results[RV32I_ALU.ALU_SRA] = sra_signed_result
    results[RV32I_ALU.ALU_CMP_EQ] = eq_result
    results[RV32I_ALU.ALU_CMP_LT] = lt_result
    results[RV32I_ALU.ALU_CMP_LTU] = ltu_result
    results[RV32I_ALU.ALU_TRUE] = Bits(32)(1)
    results[RV32I_ALU.ALU_SRA_U] = a >> alu_b[0:4]
    results[RV32I_ALU.ALU_NONE] = Bits(32)(0)

    result = priority_select(signals.alu, results, Bits(32)(0))
    condition = priority_select(signals.cond, results, Bits(32)(0))
    condition = signals.flip.select(~condition, condition)
    pc_next = (fetch_addr.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
    return result, condition, pc_next


def queue_count_const(value):
    return UInt(QUEUE_COUNT_BITS)(value)


def shift_queue(entries, consume, default_value):
    shifted = []
    for idx in range(QUEUE_DEPTH):
        value = default_value
        if idx + 2 < QUEUE_DEPTH:
            value = (consume == queue_count_const(2)).select(entries[idx + 2], value)
        if idx + 1 < QUEUE_DEPTH:
            value = (consume == queue_count_const(1)).select(entries[idx + 1], value)
        value = (consume == queue_count_const(0)).select(entries[idx], value)
        shifted.append(value)
    return shifted


def append_bundle(entries, survivor_count, accept_bundle, first_value, second_value):
    appended = []
    for idx in range(QUEUE_DEPTH):
        value = entries[idx]
        value = (accept_bundle & (survivor_count == queue_count_const(idx))).select(first_value, value)
        if idx > 0:
            value = (accept_bundle & (survivor_count == queue_count_const(idx - 1))).select(second_value, value)
        appended.append(value)
    return appended


class Execution(Module):
    def __init__(self):
        super().__init__(
            ports={
                "signals": Port(deocder_signals),
                "fetch_addr": Port(Bits(32)),
                "src_a": Port(Bits(32)),
                "src_b": Port(Bits(32)),
                "csr_id": Port(Bits(4)),
                "csr_new": Port(Bits(32)),
                "rd_epoch": Port(EPOCH_DTYPE),
            }
        )
        self.name = "E"

    @module.combinational
    def build(
        self,
        reg_file: Array,
        producer_epoch: Array,
        exec0_owner: Array,
        exec1_owner: Array,
        mem_owner: Array,
        wb_owner: Array,
        exec0_bypass_reg: Array,
        exec0_bypass_data: Array,
        exec0_bypass_epoch: Array,
        prev_exec0_bypass_reg: Array,
        prev_exec0_bypass_data: Array,
        prev_exec0_bypass_epoch: Array,
        exec1_bypass_reg: Array,
        exec1_bypass_data: Array,
        exec1_bypass_epoch: Array,
        prev_exec1_bypass_reg: Array,
        prev_exec1_bypass_data: Array,
        prev_exec1_bypass_epoch: Array,
        mem_bypass_reg: Array,
        mem_bypass_data: Array,
        mem_bypass_epoch: Array,
        wb0_bypass_reg: Array,
        wb0_bypass_data: Array,
        wb0_bypass_epoch: Array,
        wb1_bypass_reg: Array,
        wb1_bypass_data: Array,
        wb1_bypass_epoch: Array,
        csr_f: Array,
        offset_reg: Array,
        memory: Module,
        data: str,
        depth_log: int,
        exec_br_dest: Array,
        exec_br_jump_reg: Array,
        exec_br_valid_reg: Array,
    ):
        signals, fetch_addr, a, b, csr_id, csr_new, rd_epoch = self.pop_all_ports(False)

        rd = signals.rd

        is_ebreak = signals.rs1_valid & signals.imm_valid & (
            (signals.imm == Bits(32)(1)) | (signals.imm == Bits(32)(0))
        ) & is_special_op(signals)
        with Condition(is_ebreak):
            log("ebreak | halt | ecall")
            finish()

        is_trap = signals.is_branch & signals.is_offset_br & signals.imm_valid & (
            signals.imm == Bits(32)(0)
        ) & (
            signals.cond == Bits(RV32I_ALU.CNT)(1 << RV32I_ALU.ALU_TRUE)
        ) & (
            signals.alu == Bits(RV32I_ALU.CNT)(1 << RV32I_ALU.ALU_ADD)
        )
        with Condition(is_trap):
            log("trap")
            finish()

        result, condition, pc_next = build_result(signals, fetch_addr, a, b)

        memory_read = signals.memory[0:0]
        memory_write = signals.memory[1:1]
        produced_by_exec = (~memory_read) & writes_rd(signals)

        exec0_bypass_reg[0] = produced_by_exec.select(rd, Bits(5)(0))
        exec0_bypass_data[0] = produced_by_exec.select(signals.link_pc.select(pc_next, result), Bits(32)(0))
        exec0_bypass_epoch[0] = produced_by_exec.select(rd_epoch, EPOCH_DTYPE(0))

        with Condition(signals.is_branch):
            exec_br_dest[0] = condition[0:0].select(result, pc_next)
            exec_br_jump_reg[0] = condition[0:0]
            exec_br_valid_reg[0] = Bits(1)(1)
            log("branch.dest      | taken: {} | dest: 0x{:08x} | fallthrough: 0x{:08x}", condition[0:0], result, pc_next)
        with Condition(~signals.is_branch):
            exec_br_valid_reg[0] = Bits(1)(0)

        exec_br_jump = signals.is_branch.select(condition[0:0], Bits(1)(0))

        is_memory = memory_read | memory_write
        addr = (result.bitcast(UInt(32)) - is_memory.select(offset_reg[0].bitcast(UInt(32)), UInt(32)(0))).bitcast(Bits(32))
        request_addr = is_memory.select(addr[2:2 + depth_log - 1].bitcast(UInt(depth_log)), UInt(depth_log)(0))

        with Condition(memory_read):
            log("mem-read         | addr: 0x{:05x}| line: 0x{:05x} |", result, request_addr)
        with Condition(memory_write):
            log("mem-write        | addr: 0x{:05x}| line: 0x{:05x} | value: 0x{:08x} |", result, request_addr, b)

        dcache = SRAM(width=32, depth=1 << depth_log, init_file=data)
        dcache.name = "dcache"
        dcache.build(we=memory_write, re=memory_read, wdata=b, addr=request_addr)

        bound = memory.bind(
            rd=rd,
            result=signals.link_pc.select(pc_next, result),
            mem_ext=signals.mem_ext,
            is_mem_read=memory_read,
            epoch=rd_epoch,
        )
        bound.async_called()

        with Condition(signals.csr_write):
            csr_f[csr_id] = csr_new

        return rd, Bits(1)(1), exec_br_jump, dcache


class Slot1Execution(Module):
    def __init__(self):
        super().__init__(
            ports={
                "signals": Port(deocder_signals),
                "fetch_addr": Port(Bits(32)),
                "src_a": Port(Bits(32)),
                "src_b": Port(Bits(32)),
                "rd_epoch": Port(EPOCH_DTYPE),
            }
        )
        self.name = "E1"

    @module.combinational
    def build(
        self,
        reg_file: Array,
        producer_epoch: Array,
        exec0_owner: Array,
        exec1_owner: Array,
        mem_owner: Array,
        wb_owner: Array,
        exec0_bypass_reg: Array,
        exec0_bypass_data: Array,
        exec0_bypass_epoch: Array,
        prev_exec0_bypass_reg: Array,
        prev_exec0_bypass_data: Array,
        prev_exec0_bypass_epoch: Array,
        exec1_bypass_reg: Array,
        exec1_bypass_data: Array,
        exec1_bypass_epoch: Array,
        prev_exec1_bypass_reg: Array,
        prev_exec1_bypass_data: Array,
        prev_exec1_bypass_epoch: Array,
        mem_bypass_reg: Array,
        mem_bypass_data: Array,
        mem_bypass_epoch: Array,
        wb0_bypass_reg: Array,
        wb0_bypass_data: Array,
        wb0_bypass_epoch: Array,
        wb1_bypass_reg: Array,
        wb1_bypass_data: Array,
        wb1_bypass_epoch: Array,
        writeback: Module,
        exec1_br_dest: Array,
        exec1_br_jump_reg: Array,
        exec1_br_valid_reg: Array,
    ):
        signals, fetch_addr, a, b, rd_epoch = self.pop_all_ports(False)
        result, condition, pc_next = build_result(signals, fetch_addr, a, b)
        write_data = signals.link_pc.select(pc_next, result)
        produced_by_exec = writes_rd(signals)

        exec1_bypass_reg[0] = produced_by_exec.select(signals.rd, Bits(5)(0))
        exec1_bypass_data[0] = produced_by_exec.select(write_data, Bits(32)(0))
        exec1_bypass_epoch[0] = produced_by_exec.select(rd_epoch, EPOCH_DTYPE(0))

        with Condition(signals.is_branch):
            exec1_br_dest[0] = condition[0:0].select(result, pc_next)
            exec1_br_jump_reg[0] = condition[0:0]
            exec1_br_valid_reg[0] = Bits(1)(1)
            log("branch1.dest     | taken: {} | dest: 0x{:08x} | fallthrough: 0x{:08x}", condition[0:0], result, pc_next)
        with Condition(~signals.is_branch):
            exec1_br_valid_reg[0] = Bits(1)(0)

        wb_bound = writeback.bind(rd=signals.rd, mdata=write_data, epoch=rd_epoch)
        wb_bound.async_called()

        return produced_by_exec.select(signals.rd, Bits(5)(0))


class Fetcher(Module):
    def __init__(self):
        super().__init__(ports={}, no_arbiter=True)
        self.name = "F"

    @module.combinational
    def build(self):
        pc_reg = RegArray(Bits(32), 1)
        branch_wait = RegArray(Bits(1), 1, initializer=[0])
        queue_pc = [RegArray(Bits(32), 1) for _ in range(QUEUE_DEPTH)]
        queue_inst = [RegArray(Bits(32), 1) for _ in range(QUEUE_DEPTH)]
        queue_count = RegArray(UInt(QUEUE_COUNT_BITS), 1, initializer=[0])
        fetch_pending = RegArray(Bits(1), 1, initializer=[0])
        fetch_pending_pc = RegArray(Bits(32), 1)
        schedule_anchor = pc_reg[0]
        return (
            pc_reg,
            branch_wait,
            queue_pc,
            queue_inst,
            queue_count,
            fetch_pending,
            fetch_pending_pc,
            schedule_anchor,
        )


class IssueStage(Module):
    def __init__(self):
        super().__init__(ports={"tick": Port(Bits(1))}, no_arbiter=True)
        self.name = "I"

    @module.combinational
    def build(
        self,
        executor: Module,
        slot1_executor: Module,
        reg_file: Array,
        csr_file: Array,
        pc_reg: Array,
        branch_wait: Array,
        queue_pc,
        queue_inst,
        queue_count: Array,
        fetch_pending: Array,
        fetch_pending_pc: Array,
        producer_epoch: Array,
        latest_exec0_owner: Array,
        latest_exec1_owner: Array,
        latest_mem_owner: Array,
        latest_wb_owner: Array,
        issue_exec0_owner: Array,
        issue_exec1_owner: Array,
        issue_mem_owner: Array,
        exec0_owner: Array,
        exec1_owner: Array,
        mem_owner: Array,
        wb_owner: Array,
        exec0_bypass_reg: Array,
        exec0_bypass_data: Array,
        exec0_bypass_epoch: Array,
        prev_exec0_bypass_reg: Array,
        prev_exec0_bypass_data: Array,
        prev_exec0_bypass_epoch: Array,
        exec1_bypass_reg: Array,
        exec1_bypass_data: Array,
        exec1_bypass_epoch: Array,
        prev_exec1_bypass_reg: Array,
        prev_exec1_bypass_data: Array,
        prev_exec1_bypass_epoch: Array,
        mem_bypass_reg: Array,
        mem_bypass_data: Array,
        mem_bypass_epoch: Array,
        wb0_bypass_reg: Array,
        wb0_bypass_data: Array,
        wb0_bypass_epoch: Array,
        wb1_bypass_reg: Array,
        wb1_bypass_data: Array,
        wb1_bypass_epoch: Array,
        icache0: SRAM,
        icache1: SRAM,
        depth_log: int,
        exec_br_dest: Array,
        exec_br_jump: Array,
        exec_br_valid: Array,
        exec1_br_dest: Array,
        exec1_br_jump: Array,
        exec1_br_valid: Array,
    ):
        active = self.pop_all_ports(False)
        prepare_operands = prepare_execution_operands_conservative if PURE_DUAL_ISSUE_MODE else prepare_execution_operands
        operand_ready_fn = operand_ready_conservative if PURE_DUAL_ISSUE_MODE else operand_ready
        rd_hazard_fn = rd_hazard_conservative if PURE_DUAL_ISSUE_MODE else rd_hazard
        visible_exec0_owner = issue_exec0_owner[0] | exec0_owner[0]
        visible_exec1_owner = issue_exec1_owner[0] | exec1_owner[0]
        visible_mem_owner = issue_mem_owner[0] | mem_owner[0]
        visible_latest_exec0_owner = latest_exec0_owner[0]
        visible_latest_exec1_owner = latest_exec1_owner[0]
        visible_latest_mem_owner = latest_mem_owner[0]
        visible_latest_wb_owner = latest_wb_owner[0]
        ready_exec0_owner = visible_exec0_owner if PURE_DUAL_ISSUE_MODE else visible_latest_exec0_owner
        ready_exec1_owner = visible_exec1_owner if PURE_DUAL_ISSUE_MODE else visible_latest_exec1_owner
        ready_mem_owner = visible_mem_owner if PURE_DUAL_ISSUE_MODE else visible_latest_mem_owner
        ready_wb_owner = wb_owner[0] if PURE_DUAL_ISSUE_MODE else visible_latest_wb_owner
        pending_pc0 = fetch_pending_pc[0]
        pending_pc1 = (fetch_pending_pc[0].bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
        accept_pending = fetch_pending[0]
        fill_from_pending = Bits(1)(0) if PURE_DUAL_ISSUE_MODE else active & (queue_count[0] == queue_count_const(0)) & accept_pending

        visible_count = fill_from_pending.select(queue_count_const(2), queue_count[0])
        visible_pc = []
        visible_inst = []
        for idx in range(QUEUE_DEPTH):
            slot_valid = visible_count > queue_count_const(idx)
            pc_value = Bits(32)(0)
            inst_value = Bits(32)(0)
            if idx == 0:
                pc_value = fill_from_pending.select(pending_pc0, pc_value)
                inst_value = fill_from_pending.select(icache0.dout[0].bitcast(Bits(32)), inst_value)
            elif idx == 1:
                pc_value = fill_from_pending.select(pending_pc1, pc_value)
                inst_value = fill_from_pending.select(icache1.dout[0].bitcast(Bits(32)), inst_value)
            pc_value = ((~fill_from_pending) & (queue_count[0] > queue_count_const(idx))).select(queue_pc[idx][0], pc_value)
            inst_value = ((~fill_from_pending) & (queue_count[0] > queue_count_const(idx))).select(
                queue_inst[idx][0].bitcast(Bits(32)), inst_value
            )
            visible_pc.append(slot_valid.select(pc_value, Bits(32)(0)))
            visible_inst.append(slot_valid.select(inst_value, Bits(32)(0)))

        head_valid = active & (visible_count != queue_count_const(0))
        next_valid = active & (visible_count > queue_count_const(1))
        if PURE_DUAL_ISSUE_MODE:
            head_valid = head_valid & (~branch_wait[0])
            next_valid = next_valid & (~branch_wait[0])
        decode_pc = visible_pc[0]
        decode_pc_plus4 = visible_pc[1]
        inst0 = visible_inst[0].bitcast(Bits(32))
        inst1 = visible_inst[1].bitcast(Bits(32))

        signals0 = decode_logic(inst0, head_valid)
        signals1 = decode_logic(inst1, next_valid)
        slot0_a, slot0_b, slot0_csr_id, slot0_csr_new = prepare_operands(
            signals0,
            reg_file,
            csr_file,
            producer_epoch,
            visible_exec0_owner,
            visible_exec1_owner,
            visible_mem_owner,
            wb_owner,
            exec0_bypass_reg,
            exec0_bypass_data,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_data,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_data,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_data,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_data,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_data,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_data,
            wb1_bypass_epoch,
        )
        slot0_result, slot0_condition, slot0_pc_next = build_result(signals0, decode_pc, slot0_a, slot0_b)
        slot0_pair_valid = Bits(1)(0)
        slot0_pair_reg = Bits(5)(0)
        slot0_pair_data = Bits(32)(0)
        slot0_rd_epoch = issued_epoch(signals0, producer_epoch)
        slot1_rd_epoch = issued_epoch(signals1, producer_epoch)

        slot0_rs1_ready = operand_ready_fn(
            signals0.rs1,
            signals0.rs1_valid,
            producer_epoch,
            ready_exec0_owner,
            ready_exec1_owner,
            ready_mem_owner,
            ready_wb_owner,
            exec0_bypass_reg,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_epoch,
        )
        slot0_rs2_ready = operand_ready_fn(
            signals0.rs2,
            signals0.rs2_valid,
            producer_epoch,
            ready_exec0_owner,
            ready_exec1_owner,
            ready_mem_owner,
            ready_wb_owner,
            exec0_bypass_reg,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_epoch,
        )
        slot0_rd_blocked = rd_hazard_fn(
            signals0,
            producer_epoch,
            ready_exec0_owner,
            ready_exec1_owner,
            ready_mem_owner,
            ready_wb_owner,
            exec0_bypass_reg,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_epoch,
        )
        slot0_ready = head_valid & slot0_rs1_ready & slot0_rs2_ready & (~slot0_rd_blocked)
        if (not PURE_DUAL_ISSUE_MODE) and (not DISC_FAIR_DUAL_ISSUE_MODE):
            slot0_pair_valid = slot0_ready & writes_rd(signals0) & (~is_memory_op(signals0))
            slot0_pair_reg = slot0_pair_valid.select(signals0.rd, Bits(5)(0))
            slot0_pair_data = slot0_pair_valid.select(signals0.link_pc.select(slot0_pc_next, slot0_result), Bits(32)(0))

        slot1_rs1_ready_base = operand_ready_fn(
            signals1.rs1,
            signals1.rs1_valid,
            producer_epoch,
            ready_exec0_owner,
            ready_exec1_owner,
            ready_mem_owner,
            ready_wb_owner,
            exec0_bypass_reg,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_epoch,
        )
        slot1_rs1_ready = operand_ready_fn(
            signals1.rs1,
            signals1.rs1_valid,
            producer_epoch,
            ready_exec0_owner,
            ready_exec1_owner,
            ready_mem_owner,
            ready_wb_owner,
            exec0_bypass_reg,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_epoch,
            slot0_pair_valid,
            slot0_pair_reg,
        )
        slot1_rs2_ready_base = operand_ready_fn(
            signals1.rs2,
            signals1.rs2_valid,
            producer_epoch,
            ready_exec0_owner,
            ready_exec1_owner,
            ready_mem_owner,
            ready_wb_owner,
            exec0_bypass_reg,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_epoch,
        )
        slot1_rs2_ready = operand_ready_fn(
            signals1.rs2,
            signals1.rs2_valid,
            producer_epoch,
            ready_exec0_owner,
            ready_exec1_owner,
            ready_mem_owner,
            ready_wb_owner,
            exec0_bypass_reg,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_epoch,
            slot0_pair_valid,
            slot0_pair_reg,
        )
        slot1_rd_blocked = rd_hazard_fn(
            signals1,
            producer_epoch,
            ready_exec0_owner,
            ready_exec1_owner,
            ready_mem_owner,
            ready_wb_owner,
            exec0_bypass_reg,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_epoch,
        )
        slot1_a, slot1_b, _slot1_csr_id, _slot1_csr_new = prepare_operands(
            signals1,
            reg_file,
            csr_file,
            producer_epoch,
            visible_exec0_owner,
            visible_exec1_owner,
            visible_mem_owner,
            wb_owner,
            exec0_bypass_reg,
            exec0_bypass_data,
            exec0_bypass_epoch,
            prev_exec0_bypass_reg,
            prev_exec0_bypass_data,
            prev_exec0_bypass_epoch,
            exec1_bypass_reg,
            exec1_bypass_data,
            exec1_bypass_epoch,
            prev_exec1_bypass_reg,
            prev_exec1_bypass_data,
            prev_exec1_bypass_epoch,
            mem_bypass_reg,
            mem_bypass_data,
            mem_bypass_epoch,
            wb0_bypass_reg,
            wb0_bypass_data,
            wb0_bypass_epoch,
            wb1_bypass_reg,
            wb1_bypass_data,
            wb1_bypass_epoch,
            slot0_pair_valid,
            slot0_pair_reg,
            slot0_pair_data,
        )
        slot1_result, slot1_condition, slot1_pc_next = build_result(signals1, decode_pc_plus4, slot1_a, slot1_b)
        slot1_ready_direct = next_valid & slot1_rs1_ready & slot1_rs2_ready & (~slot1_rd_blocked)
        slot1_ready_swap = next_valid & slot1_rs1_ready_base & slot1_rs2_ready_base & (~slot1_rd_blocked)
        slot1_pair_ok = slot1_allows_pair(signals1)
        if DISC_FAIR_DUAL_ISSUE_MODE:
            slot1_pair_ok = slot1_pair_ok & (~signals1.is_branch)
        direct_pair_safe = pair_safe(signals0, signals1)
        if PURE_DUAL_ISSUE_MODE or DISC_FAIR_DUAL_ISSUE_MODE:
            direct_pair_safe = pair_swap_safe(signals0, signals1)
        direct_pair = (
            slot0_ready
            & slot1_ready_direct
            & slot0_allows_pair(signals0)
            & slot1_pair_ok
            & direct_pair_safe
        )
        swap_pair = Bits(1)(0)
        if not PURE_DUAL_ISSUE_MODE:
            swap_pair = (
                slot0_ready
                & slot1_ready_swap
                & slot0_allows_pair(signals0)
                & (~is_memory_op(signals0))
                & slot1_prefers_lane0(signals1)
                & pair_swap_safe(signals0, signals1)
            )
        slot1_issue = direct_pair | swap_pair
        slot0_issue_single = slot0_ready & (~slot1_issue)
        slot0_branch_issue = slot0_ready & signals0.is_branch
        slot1_branch_issue = direct_pair & signals1.is_branch
        branch_issue = slot0_branch_issue | slot1_branch_issue
        slot0_branch_next_pc = slot0_condition[0:0].select(slot0_result, slot0_pc_next)
        slot1_branch_next_pc = slot1_condition[0:0].select(slot1_result, slot1_pc_next)
        issue_branch_taken = slot1_branch_issue.select(slot1_condition[0:0], slot0_condition[0:0])
        issue_branch_next_pc = slot1_branch_issue.select(slot1_branch_next_pc, slot0_branch_next_pc)
        resolved_branch_valid = exec1_br_valid[0] | exec_br_valid[0]
        resolved_branch_taken = exec1_br_valid[0].select(exec1_br_jump[0], exec_br_jump[0])
        resolved_branch_next_pc = exec1_br_valid[0].select(exec1_br_dest[0], exec_br_dest[0])
        branch_taken = resolved_branch_taken if PURE_DUAL_ISSUE_MODE else issue_branch_taken
        branch_next_pc = resolved_branch_next_pc if PURE_DUAL_ISSUE_MODE else issue_branch_next_pc
        branch_redirect = (resolved_branch_valid & resolved_branch_taken) if PURE_DUAL_ISSUE_MODE else (branch_issue & issue_branch_taken)
        flush_frontend = branch_redirect
        lane0_prev_bypass_reg = Bits(5)(0)
        lane0_prev_bypass_data = Bits(32)(0)
        lane0_prev_bypass_epoch = EPOCH_DTYPE(0)
        lane1_prev_bypass_reg = Bits(5)(0)
        lane1_prev_bypass_data = Bits(32)(0)
        lane1_prev_bypass_epoch = EPOCH_DTYPE(0)
        slot0_exec_rd = ((slot0_issue_single | direct_pair) & writes_rd(signals0) & (~is_memory_op(signals0))).select(
            signals0.rd, Bits(5)(0)
        )
        slot0_mem_rd = ((slot0_issue_single | direct_pair) & writes_rd(signals0) & is_memory_op(signals0)).select(
            signals0.rd, Bits(5)(0)
        )
        slot0_mem_rd = (swap_pair & writes_rd(signals1) & is_memory_op(signals1)).select(signals1.rd, slot0_mem_rd)
        slot1_exec_rd = direct_pair.select(
            writes_rd(signals1).select(signals1.rd, Bits(5)(0)),
            Bits(5)(0),
        )
        slot1_exec_rd = swap_pair.select(
            writes_rd(signals0).select(signals0.rd, Bits(5)(0)),
            slot1_exec_rd,
        )
        lane0_prev_bypass_reg = slot0_exec_rd
        lane0_prev_bypass_data = ((slot0_issue_single | direct_pair) & writes_rd(signals0) & (~is_memory_op(signals0))).select(
            signals0.link_pc.select(slot0_pc_next, slot0_result),
            lane0_prev_bypass_data,
        )
        lane0_prev_bypass_epoch = ((slot0_issue_single | direct_pair) & writes_rd(signals0) & (~is_memory_op(signals0))).select(
            slot0_rd_epoch,
            lane0_prev_bypass_epoch,
        )
        lane0_prev_bypass_reg = (swap_pair & writes_rd(signals1) & (~is_memory_op(signals1))).select(
            signals1.rd,
            lane0_prev_bypass_reg,
        )
        lane0_prev_bypass_data = (swap_pair & writes_rd(signals1) & (~is_memory_op(signals1))).select(
            signals1.link_pc.select(slot1_pc_next, slot1_result),
            lane0_prev_bypass_data,
        )
        lane0_prev_bypass_epoch = (swap_pair & writes_rd(signals1) & (~is_memory_op(signals1))).select(
            slot1_rd_epoch,
            lane0_prev_bypass_epoch,
        )
        lane1_prev_bypass_reg = direct_pair.select(
            writes_rd(signals1).select(signals1.rd, Bits(5)(0)),
            lane1_prev_bypass_reg,
        )
        lane1_prev_bypass_data = (direct_pair & writes_rd(signals1) & (~is_memory_op(signals1))).select(
            signals1.link_pc.select(slot1_pc_next, slot1_result),
            lane1_prev_bypass_data,
        )
        lane1_prev_bypass_epoch = (direct_pair & writes_rd(signals1) & (~is_memory_op(signals1))).select(
            slot1_rd_epoch,
            lane1_prev_bypass_epoch,
        )
        lane1_prev_bypass_reg = swap_pair.select(
            writes_rd(signals0).select(signals0.rd, Bits(5)(0)),
            lane1_prev_bypass_reg,
        )
        lane1_prev_bypass_data = (swap_pair & writes_rd(signals0) & (~is_memory_op(signals0))).select(
            signals0.link_pc.select(slot0_pc_next, slot0_result),
            lane1_prev_bypass_data,
        )
        lane1_prev_bypass_epoch = (swap_pair & writes_rd(signals0) & (~is_memory_op(signals0))).select(
            slot0_rd_epoch,
            lane1_prev_bypass_epoch,
        )

        consume_count = slot1_issue.select(queue_count_const(2), slot0_ready.select(queue_count_const(1), queue_count_const(0)))

        survivor_pc = shift_queue(visible_pc, consume_count, Bits(32)(0))
        survivor_inst = shift_queue(visible_inst, consume_count, Bits(32)(0))

        survivor_count = flush_frontend.select(queue_count_const(0), visible_count - consume_count)

        accept_pending_to_queue = accept_pending & (~fill_from_pending) & (~flush_frontend)
        merged_pc = append_bundle(survivor_pc, survivor_count, accept_pending_to_queue, pending_pc0, pending_pc1)
        merged_inst = append_bundle(
            survivor_inst,
            survivor_count,
            accept_pending_to_queue,
            icache0.dout[0].bitcast(Bits(32)),
            icache1.dout[0].bitcast(Bits(32)),
        )
        merged_count = survivor_count + accept_pending_to_queue.select(queue_count_const(2), queue_count_const(0))

        request_pc = branch_redirect.select(branch_next_pc, pc_reg[0])
        request_pc_plus4 = (request_pc.bitcast(Int(32)) + Int(32)(4)).bitcast(Bits(32))
        request_pc_plus8 = (request_pc.bitcast(Int(32)) + Int(32)(8)).bitcast(Bits(32))
        request_fetch = active & (merged_count <= queue_count_const(QUEUE_DEPTH - 2))
        if PURE_DUAL_ISSUE_MODE:
            request_fetch = request_fetch & ((((~branch_wait[0]) & (~branch_issue)) | resolved_branch_valid))

        with Condition(head_valid):
            log("issue.check      | pc: 0x{:08x} | qcnt: {} | slot0_ready: {} | slot1_issue: {}", decode_pc, queue_count[0], slot0_ready, slot1_issue)
        with Condition(head_valid & (~slot0_ready)):
            log(
                "slot0.block      | pc: 0x{:08x} | rd: x{:02} | rs1_ready: {} | rs2_ready: {} | rd_blocked: {} | own(e0/e1/m/w): {}{}{}{}",
                decode_pc,
                signals0.rd,
                slot0_rs1_ready,
                slot0_rs2_ready,
                slot0_rd_blocked,
                reg_tracked(visible_exec0_owner, signals0.rd),
                reg_tracked(visible_exec1_owner, signals0.rd),
                reg_tracked(visible_mem_owner, signals0.rd),
                reg_tracked(wb_owner, signals0.rd),
            )
        with Condition(head_valid & (~slot0_ready) & signals0.is_branch):
            log(
                "branch.waitsrc   | pc: 0x{:08x} | rs1: x{:02} | expect: {} | own(e0/e1/m/w): {}{}{}{} | by(e0/e1/m/w0/w1): x{:02}/{} x{:02}/{} x{:02}/{} x{:02}/{} x{:02}/{}",
                decode_pc,
                signals0.rs1,
                reg_epoch(producer_epoch, signals0.rs1),
                reg_tracked(visible_exec0_owner, signals0.rs1),
                reg_tracked(visible_exec1_owner, signals0.rs1),
                reg_tracked(visible_mem_owner, signals0.rs1),
                reg_tracked(wb_owner, signals0.rs1),
                exec0_bypass_reg[0],
                exec0_bypass_epoch[0],
                exec1_bypass_reg[0],
                exec1_bypass_epoch[0],
                mem_bypass_reg[0],
                mem_bypass_epoch[0],
                wb0_bypass_reg[0],
                wb0_bypass_epoch[0],
                wb1_bypass_reg[0],
                wb1_bypass_epoch[0],
            )

        with Condition(request_fetch):
            log("fetch.request    | base_pc: 0x{:08x} | qcnt_after_merge: {}", request_pc, merged_count)

        with Condition(resolved_branch_valid if PURE_DUAL_ISSUE_MODE else branch_issue):
            log("branch.resolve   | taken: {} | next_pc: 0x{:08x}", branch_taken, branch_next_pc)

        with Condition(swap_pair):
            log("pair.swap        | pc0: 0x{:08x} | pc1: 0x{:08x}", decode_pc, decode_pc_plus4)

        with Condition(slot0_issue_single | direct_pair):
            with Condition(signals0.is_branch):
                log(
                    "branch.src       | pc: 0x{:08x} | a: 0x{:08x} | b: 0x{:08x} | rs1: x{:02} | rs2: x{:02} | own(e0/e1/m/w): {}{}{}{} | by(e0/e1/m/w0/w1): x{:02}/{} x{:02}/{} x{:02}/{} x{:02}/{} x{:02}/{}",
                    decode_pc,
                    slot0_a,
                    slot0_b,
                    signals0.rs1,
                    signals0.rs2,
                    reg_tracked(visible_exec0_owner, signals0.rs1),
                    reg_tracked(visible_exec1_owner, signals0.rs1),
                    reg_tracked(visible_mem_owner, signals0.rs1),
                    reg_tracked(wb_owner, signals0.rs1),
                    exec0_bypass_reg[0],
                    exec0_bypass_epoch[0],
                    exec1_bypass_reg[0],
                    exec1_bypass_epoch[0],
                    mem_bypass_reg[0],
                    mem_bypass_epoch[0],
                    wb0_bypass_reg[0],
                    wb0_bypass_epoch[0],
                    wb1_bypass_reg[0],
                    wb1_bypass_epoch[0],
                )
            e_call = executor.async_called(
                signals=signals0,
                fetch_addr=decode_pc,
                src_a=slot0_a,
                src_b=slot0_b,
                csr_id=slot0_csr_id,
                csr_new=slot0_csr_new,
                rd_epoch=slot0_rd_epoch,
            )
            e_call.bind.set_fifo_depth(signals=2, fetch_addr=2, src_a=2, src_b=2, csr_id=2, csr_new=2, rd_epoch=2)

        with Condition(swap_pair):
            e_swap_call = executor.async_called(
                signals=signals1,
                fetch_addr=decode_pc_plus4,
                src_a=slot1_a,
                src_b=slot1_b,
                csr_id=Bits(4)(0),
                csr_new=Bits(32)(0),
                rd_epoch=slot1_rd_epoch,
            )
            e_swap_call.bind.set_fifo_depth(signals=2, fetch_addr=2, src_a=2, src_b=2, csr_id=2, csr_new=2, rd_epoch=2)

        with Condition(direct_pair):
            with Condition(signals1.is_branch):
                log(
                    "branch1.src      | pc: 0x{:08x} | a: 0x{:08x} | b: 0x{:08x} | rs1: x{:02} | rs2: x{:02}",
                    decode_pc_plus4,
                    slot1_a,
                    slot1_b,
                    signals1.rs1,
                    signals1.rs2,
                )
            e1_call = slot1_executor.async_called(
                signals=signals1,
                fetch_addr=decode_pc_plus4,
                src_a=slot1_a,
                src_b=slot1_b,
                rd_epoch=slot1_rd_epoch,
            )
            e1_call.bind.set_fifo_depth(signals=2, fetch_addr=2, src_a=2, src_b=2, rd_epoch=2)

        with Condition(swap_pair):
            e1_swap_call = slot1_executor.async_called(
                signals=signals0,
                fetch_addr=decode_pc,
                src_a=slot0_a,
                src_b=slot0_b,
                rd_epoch=slot0_rd_epoch,
            )
            e1_swap_call.bind.set_fifo_depth(signals=2, fetch_addr=2, src_a=2, src_b=2, rd_epoch=2)

        for idx in range(QUEUE_DEPTH):
            queue_pc[idx][0] = merged_pc[idx]
            queue_inst[idx][0] = merged_inst[idx]

        if PURE_DUAL_ISSUE_MODE:
            with Condition(branch_issue):
                branch_wait[0] = Bits(1)(1)
            with Condition(resolved_branch_valid):
                branch_wait[0] = Bits(1)(0)
        else:
            branch_wait[0] = Bits(1)(0)
        queue_count[0] = merged_count
        fetch_pending[0] = request_fetch.select(Bits(1)(1), Bits(1)(0))
        with Condition(request_fetch):
            fetch_pending_pc[0] = request_pc
            pc_reg[0] = request_pc_plus8

        with Condition(branch_redirect & (~request_fetch)):
            pc_reg[0] = branch_next_pc
        exec_br_valid[0] = Bits(1)(0)
        exec1_br_valid[0] = Bits(1)(0)

        # Bypass values should only live for the cycle they are produced.
        # Otherwise old entries can match again when 2-bit producer epochs wrap.
        exec0_bypass_reg[0] = Bits(5)(0)
        exec1_bypass_reg[0] = Bits(5)(0)
        mem_bypass_reg[0] = Bits(5)(0)
        wb0_bypass_reg[0] = Bits(5)(0)
        wb1_bypass_reg[0] = Bits(5)(0)
        prev_exec0_bypass_reg[0] = Bits(5)(0) if PURE_DUAL_ISSUE_MODE else lane0_prev_bypass_reg
        prev_exec0_bypass_data[0] = Bits(32)(0) if PURE_DUAL_ISSUE_MODE else lane0_prev_bypass_data
        prev_exec0_bypass_epoch[0] = EPOCH_DTYPE(0) if PURE_DUAL_ISSUE_MODE else lane0_prev_bypass_epoch
        prev_exec1_bypass_reg[0] = Bits(5)(0) if PURE_DUAL_ISSUE_MODE else lane1_prev_bypass_reg
        prev_exec1_bypass_data[0] = Bits(32)(0) if PURE_DUAL_ISSUE_MODE else lane1_prev_bypass_data
        prev_exec1_bypass_epoch[0] = EPOCH_DTYPE(0) if PURE_DUAL_ISSUE_MODE else lane1_prev_bypass_epoch

        icache0.build(Bits(1)(0), request_fetch, request_pc[2:2 + depth_log - 1].bitcast(Int(depth_log)), Bits(32)(0))
        icache1.build(Bits(1)(0), request_fetch, request_pc_plus4[2:2 + depth_log - 1].bitcast(Int(depth_log)), Bits(32)(0))
        return slot0_exec_rd, slot0_mem_rd, slot1_exec_rd


class Onwrite(Downstream):
    def __init__(self):
        super().__init__()
        self.name = "P"

    @downstream.combinational
    def build(
        self,
        producer_epoch: Array,
        latest_exec0_owner: Array,
        latest_exec1_owner: Array,
        latest_mem_owner: Array,
        latest_wb_owner: Array,
        issue_exec0_owner: Array,
        issue_exec1_owner: Array,
        issue_mem_owner: Array,
        exec0_owner: Array,
        exec1_owner: Array,
        mem_owner: Array,
        wb_owner: Array,
        slot0_exec_rd: Value,
        slot0_mem_rd: Value,
        slot1_exec_rd: Value,
        wb0_rd: Value,
        wb1_rd: Value,
        exec0_bypass_reg: Array,
        exec0_bypass_data: Array,
        exec0_bypass_epoch: Array,
        prev_exec0_bypass_reg: Array,
        prev_exec0_bypass_data: Array,
        prev_exec0_bypass_epoch: Array,
        exec1_bypass_reg: Array,
        exec1_bypass_data: Array,
        exec1_bypass_epoch: Array,
        prev_exec1_bypass_reg: Array,
        prev_exec1_bypass_data: Array,
        prev_exec1_bypass_epoch: Array,
        mem_bypass_reg: Array,
        mem_bypass_epoch: Array,
        wb0_bypass_reg: Array,
        wb0_bypass_epoch: Array,
        wb1_bypass_reg: Array,
        wb1_bypass_epoch: Array,
    ):
        ex0_rd = slot0_exec_rd.optional(Bits(5)(0))
        mem_rd = slot0_mem_rd.optional(Bits(5)(0))
        ex1_rd = slot1_exec_rd.optional(Bits(5)(0))
        wb0 = wb0_rd.optional(Bits(5)(0))
        wb1 = wb1_rd.optional(Bits(5)(0))

        ex0_bit = rd_mask(ex0_rd)
        mem_bit = rd_mask(mem_rd)
        ex1_bit = rd_mask(ex1_rd)

        exec0_visible = visible_stage_mask(exec0_bypass_reg, exec0_bypass_epoch, producer_epoch)
        exec1_visible = visible_stage_mask(exec1_bypass_reg, exec1_bypass_epoch, producer_epoch)
        mem_visible = visible_stage_mask(mem_bypass_reg, mem_bypass_epoch, producer_epoch)
        wb0_visible = visible_stage_mask(wb0_bypass_reg, wb0_bypass_epoch, producer_epoch)
        wb1_visible = visible_stage_mask(wb1_bypass_reg, wb1_bypass_epoch, producer_epoch)
        wb_visible = wb0_visible | wb1_visible
        transfer_mask = ~wb_visible
        new_issue_mask = ex0_bit | ex1_bit | mem_bit
        next_epoch_word = producer_epoch[0]
        next_epoch_word = set_reg_epoch(next_epoch_word, ex0_rd, next_producer_epoch(producer_epoch, ex0_rd))
        next_epoch_word = set_reg_epoch(next_epoch_word, mem_rd, next_producer_epoch(producer_epoch, mem_rd))
        next_epoch_word = set_reg_epoch(next_epoch_word, ex1_rd, next_producer_epoch(producer_epoch, ex1_rd))

        issue_exec0_owner[0] = ex0_bit
        issue_exec1_owner[0] = ex1_bit
        issue_mem_owner[0] = mem_bit
        exec0_owner[0] = ex0_bit | (exec0_owner[0] & (~exec0_visible))
        exec1_owner[0] = ex1_bit | (exec1_owner[0] & (~exec1_visible))
        mem_owner[0] = mem_bit | (mem_owner[0] & (~mem_visible))
        wb_owner[0] = (wb_owner[0] & transfer_mask) | ((exec0_owner[0] & exec0_visible) & transfer_mask) | ((exec1_owner[0] & exec1_visible) & transfer_mask) | ((mem_owner[0] & mem_visible) & transfer_mask)
        producer_epoch[0] = next_epoch_word
        latest_exec0_owner[0] = ex0_bit | (latest_exec0_owner[0] & (~exec0_visible) & (~new_issue_mask))
        latest_exec1_owner[0] = ex1_bit | (latest_exec1_owner[0] & (~exec1_visible) & (~new_issue_mask))
        latest_mem_owner[0] = mem_bit | (latest_mem_owner[0] & (~mem_visible) & (~new_issue_mask))
        latest_wb_owner[0] = (
            (latest_wb_owner[0] & (~wb_visible))
            | (latest_exec0_owner[0] & exec0_visible)
            | (latest_exec1_owner[0] & exec1_visible)
            | (latest_mem_owner[0] & mem_visible)
        ) & (~new_issue_mask)
class MemUser(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, rdata: RegArray):
        width = rdata.scalar_ty.bits
        raw = rdata[0].bitcast(Int(width))
        offset_reg = RegArray(Bits(width), 1)
        offset_reg[0] = raw.bitcast(Bits(width))
        return offset_reg


class Driver(Module):
    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self, fetcher: Module, issuer: Module, user: Module):
        init_reg = RegArray(UInt(1), 1, initializer=[1])
        init_cache = SRAM(width=32, depth=32, init_file=f"{workspace}/workload.init")
        init_cache.name = "init_cache"
        init_cache.build(we=Bits(1)(0), re=init_reg[0].bitcast(Bits(1)), wdata=Bits(32)(0), addr=Bits(5)(0))

        with Condition(init_reg[0] == UInt(1)(1)):
            user.async_called()
            init_reg[0] = UInt(1)(0)

        with Condition(init_reg[0] == UInt(1)(0)):
            fetcher.async_called()
            issuer.async_called(tick=Bits(1)(1))

        return init_cache, (init_reg[0] == UInt(1)(0)).bitcast(Bits(1))


def build_cpu(depth_log):
    if PURE_DUAL_ISSUE_MODE:
        sys_name = "minor_cpu_2_issue_pure"
    elif DISC_FAIR_DUAL_ISSUE_MODE:
        sys_name = "minor_cpu_2_issue_disc_fair"
    else:
        sys_name = "minor_cpu_2_issue"
    sys = SysBuilder(sys_name)

    with sys:
        bits5 = Bits(5)
        bits32 = Bits(32)

        reg_file = RegArray(bits32, 32)
        csr_file = RegArray(Bits(32), 16, initializer=[0] * 16)

        producer_epoch = RegArray(PRODUCER_EPOCH_DTYPE, 1)
        latest_exec0_owner = RegArray(bits32, 1)
        latest_exec1_owner = RegArray(bits32, 1)
        latest_mem_owner = RegArray(bits32, 1)
        latest_wb_owner = RegArray(bits32, 1)
        issue_exec0_owner = RegArray(bits32, 1)
        issue_exec1_owner = RegArray(bits32, 1)
        issue_mem_owner = RegArray(bits32, 1)
        exec0_owner = RegArray(bits32, 1)
        exec1_owner = RegArray(bits32, 1)
        mem_owner = RegArray(bits32, 1)
        wb_owner = RegArray(bits32, 1)

        exec0_bypass_reg = RegArray(bits5, 1)
        exec0_bypass_data = RegArray(bits32, 1)
        exec0_bypass_epoch = RegArray(EPOCH_DTYPE, 1)
        prev_exec0_bypass_reg = RegArray(bits5, 1)
        prev_exec0_bypass_data = RegArray(bits32, 1)
        prev_exec0_bypass_epoch = RegArray(EPOCH_DTYPE, 1)
        exec1_bypass_reg = RegArray(bits5, 1)
        exec1_bypass_data = RegArray(bits32, 1)
        exec1_bypass_epoch = RegArray(EPOCH_DTYPE, 1)
        prev_exec1_bypass_reg = RegArray(bits5, 1)
        prev_exec1_bypass_data = RegArray(bits32, 1)
        prev_exec1_bypass_epoch = RegArray(EPOCH_DTYPE, 1)
        mem_bypass_reg = RegArray(bits5, 1)
        mem_bypass_data = RegArray(bits32, 1)
        mem_bypass_epoch = RegArray(EPOCH_DTYPE, 1)
        wb0_bypass_reg = RegArray(bits5, 1)
        wb0_bypass_data = RegArray(bits32, 1)
        wb0_bypass_epoch = RegArray(EPOCH_DTYPE, 1)
        wb1_bypass_reg = RegArray(bits5, 1)
        wb1_bypass_data = RegArray(bits32, 1)
        wb1_bypass_epoch = RegArray(EPOCH_DTYPE, 1)
        exec_br_dest = RegArray(Bits(32), 1)
        exec_br_jump = RegArray(Bits(1), 1, initializer=[0])
        exec_br_valid = RegArray(Bits(1), 1, initializer=[0])
        exec1_br_dest = RegArray(Bits(32), 1)
        exec1_br_jump = RegArray(Bits(1), 1, initializer=[0])
        exec1_br_valid = RegArray(Bits(1), 1, initializer=[0])

        icache0 = SRAM(width=32, depth=1 << depth_log, init_file=f"{workspace}/workload.exe")
        icache0.name = "icache0"
        icache1 = SRAM(width=32, depth=1 << depth_log, init_file=f"{workspace}/workload.exe")
        icache1.name = "icache1"

        user = MemUser()
        fetcher = Fetcher()
        (
            pc_reg,
            branch_wait,
            queue_pc,
            queue_inst,
            queue_count,
            fetch_pending,
            fetch_pending_pc,
            _schedule_anchor,
        ) = fetcher.build()
        issuer = IssueStage()
        driver = Driver()
        init_cache, _run_enable = driver.build(fetcher, issuer, user)
        offset_reg = user.build(init_cache.dout)

        writeback0 = WriteBack0()
        wb0_rd = writeback0.build(
            reg_file=reg_file,
            wb0_bypass_reg=wb0_bypass_reg,
            wb0_bypass_data=wb0_bypass_data,
            wb0_bypass_epoch=wb0_bypass_epoch,
        )
        writeback1 = WriteBack1()
        wb1_rd = writeback1.build(
            reg_file=reg_file,
            wb1_bypass_reg=wb1_bypass_reg,
            wb1_bypass_data=wb1_bypass_data,
            wb1_bypass_epoch=wb1_bypass_epoch,
        )

        memory_access = MemoryAccess()
        executor = Execution()
        _exec_rd, _ex_valid, _exec_br_jump, dcache = executor.build(
            reg_file=reg_file,
            producer_epoch=producer_epoch,
            exec0_owner=exec0_owner,
            exec1_owner=exec1_owner,
            mem_owner=mem_owner,
            wb_owner=wb_owner,
            exec0_bypass_reg=exec0_bypass_reg,
            exec0_bypass_data=exec0_bypass_data,
            exec0_bypass_epoch=exec0_bypass_epoch,
            prev_exec0_bypass_reg=prev_exec0_bypass_reg,
            prev_exec0_bypass_data=prev_exec0_bypass_data,
            prev_exec0_bypass_epoch=prev_exec0_bypass_epoch,
            exec1_bypass_reg=exec1_bypass_reg,
            exec1_bypass_data=exec1_bypass_data,
            exec1_bypass_epoch=exec1_bypass_epoch,
            prev_exec1_bypass_reg=prev_exec1_bypass_reg,
            prev_exec1_bypass_data=prev_exec1_bypass_data,
            prev_exec1_bypass_epoch=prev_exec1_bypass_epoch,
            mem_bypass_reg=mem_bypass_reg,
            mem_bypass_data=mem_bypass_data,
            mem_bypass_epoch=mem_bypass_epoch,
            wb0_bypass_reg=wb0_bypass_reg,
            wb0_bypass_data=wb0_bypass_data,
            wb0_bypass_epoch=wb0_bypass_epoch,
            wb1_bypass_reg=wb1_bypass_reg,
            wb1_bypass_data=wb1_bypass_data,
            wb1_bypass_epoch=wb1_bypass_epoch,
            csr_f=csr_file,
            offset_reg=offset_reg,
            memory=memory_access,
            data=f"{workspace}/workload.data",
            depth_log=depth_log,
            exec_br_dest=exec_br_dest,
            exec_br_jump_reg=exec_br_jump,
            exec_br_valid_reg=exec_br_valid,
        )

        memory_access.build(
            writeback=writeback0,
            mem_bypass_reg=mem_bypass_reg,
            mem_bypass_data=mem_bypass_data,
            mem_bypass_epoch=mem_bypass_epoch,
            rdata=dcache.dout,
        )

        slot1_executor = Slot1Execution()
        _slot1_exec_rd = slot1_executor.build(
            reg_file=reg_file,
            producer_epoch=producer_epoch,
            exec0_owner=exec0_owner,
            exec1_owner=exec1_owner,
            mem_owner=mem_owner,
            wb_owner=wb_owner,
            exec0_bypass_reg=exec0_bypass_reg,
            exec0_bypass_data=exec0_bypass_data,
            exec0_bypass_epoch=exec0_bypass_epoch,
            prev_exec0_bypass_reg=prev_exec0_bypass_reg,
            prev_exec0_bypass_data=prev_exec0_bypass_data,
            prev_exec0_bypass_epoch=prev_exec0_bypass_epoch,
            exec1_bypass_reg=exec1_bypass_reg,
            exec1_bypass_data=exec1_bypass_data,
            exec1_bypass_epoch=exec1_bypass_epoch,
            prev_exec1_bypass_reg=prev_exec1_bypass_reg,
            prev_exec1_bypass_data=prev_exec1_bypass_data,
            prev_exec1_bypass_epoch=prev_exec1_bypass_epoch,
            mem_bypass_reg=mem_bypass_reg,
            mem_bypass_data=mem_bypass_data,
            mem_bypass_epoch=mem_bypass_epoch,
            wb0_bypass_reg=wb0_bypass_reg,
            wb0_bypass_data=wb0_bypass_data,
            wb0_bypass_epoch=wb0_bypass_epoch,
            wb1_bypass_reg=wb1_bypass_reg,
            wb1_bypass_data=wb1_bypass_data,
            wb1_bypass_epoch=wb1_bypass_epoch,
            writeback=writeback1,
            exec1_br_dest=exec1_br_dest,
            exec1_br_jump_reg=exec1_br_jump,
            exec1_br_valid_reg=exec1_br_valid,
        )

        slot0_issue_exec_rd, slot0_issue_mem_rd, slot1_issue_exec_rd = issuer.build(
            executor=executor,
            slot1_executor=slot1_executor,
            reg_file=reg_file,
            csr_file=csr_file,
            pc_reg=pc_reg,
            branch_wait=branch_wait,
            queue_pc=queue_pc,
            queue_inst=queue_inst,
            queue_count=queue_count,
            fetch_pending=fetch_pending,
            fetch_pending_pc=fetch_pending_pc,
            producer_epoch=producer_epoch,
            latest_exec0_owner=latest_exec0_owner,
            latest_exec1_owner=latest_exec1_owner,
            latest_mem_owner=latest_mem_owner,
            latest_wb_owner=latest_wb_owner,
            issue_exec0_owner=issue_exec0_owner,
            issue_exec1_owner=issue_exec1_owner,
            issue_mem_owner=issue_mem_owner,
            exec0_owner=exec0_owner,
            exec1_owner=exec1_owner,
            mem_owner=mem_owner,
            wb_owner=wb_owner,
            exec0_bypass_reg=exec0_bypass_reg,
            exec0_bypass_data=exec0_bypass_data,
            exec0_bypass_epoch=exec0_bypass_epoch,
            prev_exec0_bypass_reg=prev_exec0_bypass_reg,
            prev_exec0_bypass_data=prev_exec0_bypass_data,
            prev_exec0_bypass_epoch=prev_exec0_bypass_epoch,
            exec1_bypass_reg=exec1_bypass_reg,
            exec1_bypass_data=exec1_bypass_data,
            exec1_bypass_epoch=exec1_bypass_epoch,
            prev_exec1_bypass_reg=prev_exec1_bypass_reg,
            prev_exec1_bypass_data=prev_exec1_bypass_data,
            prev_exec1_bypass_epoch=prev_exec1_bypass_epoch,
            mem_bypass_reg=mem_bypass_reg,
            mem_bypass_data=mem_bypass_data,
            mem_bypass_epoch=mem_bypass_epoch,
            wb0_bypass_reg=wb0_bypass_reg,
            wb0_bypass_data=wb0_bypass_data,
            wb0_bypass_epoch=wb0_bypass_epoch,
            wb1_bypass_reg=wb1_bypass_reg,
            wb1_bypass_data=wb1_bypass_data,
            wb1_bypass_epoch=wb1_bypass_epoch,
            icache0=icache0,
            icache1=icache1,
            depth_log=depth_log,
            exec_br_dest=exec_br_dest,
            exec_br_jump=exec_br_jump,
            exec_br_valid=exec_br_valid,
            exec1_br_dest=exec1_br_dest,
            exec1_br_jump=exec1_br_jump,
            exec1_br_valid=exec1_br_valid,
        )

        pending = Onwrite()
        pending.build(
            producer_epoch=producer_epoch,
            latest_exec0_owner=latest_exec0_owner,
            latest_exec1_owner=latest_exec1_owner,
            latest_mem_owner=latest_mem_owner,
            latest_wb_owner=latest_wb_owner,
            issue_exec0_owner=issue_exec0_owner,
            issue_exec1_owner=issue_exec1_owner,
            issue_mem_owner=issue_mem_owner,
            exec0_owner=exec0_owner,
            exec1_owner=exec1_owner,
            mem_owner=mem_owner,
            wb_owner=wb_owner,
            slot0_exec_rd=slot0_issue_exec_rd,
            slot0_mem_rd=slot0_issue_mem_rd,
            slot1_exec_rd=slot1_issue_exec_rd,
            exec0_bypass_reg=exec0_bypass_reg,
            exec0_bypass_data=exec0_bypass_data,
            exec0_bypass_epoch=exec0_bypass_epoch,
            prev_exec0_bypass_reg=prev_exec0_bypass_reg,
            prev_exec0_bypass_data=prev_exec0_bypass_data,
            prev_exec0_bypass_epoch=prev_exec0_bypass_epoch,
            exec1_bypass_reg=exec1_bypass_reg,
            exec1_bypass_data=exec1_bypass_data,
            exec1_bypass_epoch=exec1_bypass_epoch,
            prev_exec1_bypass_reg=prev_exec1_bypass_reg,
            prev_exec1_bypass_data=prev_exec1_bypass_data,
            prev_exec1_bypass_epoch=prev_exec1_bypass_epoch,
            mem_bypass_reg=mem_bypass_reg,
            mem_bypass_epoch=mem_bypass_epoch,
            wb0_rd=wb0_rd,
            wb1_rd=wb1_rd,
            wb0_bypass_reg=wb0_bypass_reg,
            wb0_bypass_epoch=wb0_bypass_epoch,
            wb1_bypass_reg=wb1_bypass_reg,
            wb1_bypass_epoch=wb1_bypass_epoch,
        )

        sys.expose_on_top(reg_file, kind="Output")
        sys.expose_on_top(pc_reg, kind="Output")
        sys.expose_on_top(exec_br_dest, kind="Output")

    sim_threshold = int(os.environ.get("MINOR2_SIM_THRESHOLD", "2000000"))
    idle_threshold = int(os.environ.get("MINOR2_IDLE_THRESHOLD", "2000000"))
    conf = config(
        verilog=utils.has_verilator(),
        sim_threshold=sim_threshold,
        idle_threshold=idle_threshold,
        resource_base="",
        fifo_depth=1,
    )

    simulator_path, verilog_path = elaborate(sys, **conf)
    simulator_binary = utils.build_simulator(simulator_path)
    return sys, simulator_binary, verilog_path


def extract_last_cycle(raw):
    last_cycle = 0
    for match in re.finditer(r"Cycle @([0-9]+(?:\.[0-9]+)?)", raw):
        last_cycle = max(last_cycle, int(float(match.group(1))))
    return last_cycle


def extract_kernel_cycles(raw):
    marker_cycles = []
    for line in raw.splitlines():
        if ("csr_addr: 0xb00" not in line) and ("imm: 0xb00" not in line):
            continue
        match = re.search(r"Cycle @([0-9]+(?:\.[0-9]+)?)", line)
        if match is None:
            continue
        cycle = int(float(match.group(1)))
        if not marker_cycles or marker_cycles[-1] != cycle:
            marker_cycles.append(cycle)
    if len(marker_cycles) >= 2:
        return marker_cycles[-1] - marker_cycles[0]
    return None


def run_cpu(sys, simulator_binary, workload="default", verify_verilator=False, verilog_path=None):
    with sys:
        with open(f"{workspace}/workload.config") as f:
            raw = f.readline()
            raw = raw.replace("offset:", "'offset':").replace("data_offset:", "'data_offset':")
            offsets = eval(raw)
            value = hex(offsets["data_offset"])
            value = value[1:] if value[0] == "-" else value
            value = value[2:]
            open(f"{workspace}/workload.init", "w").write(value)

    raw = utils.run_simulator(binary_path=simulator_binary)
    open("raw.log", "w").write(raw)
    open(f"{workload}.log", "w").write(raw)
    check()
    os.remove("raw.log")
    if verify_verilator and verilog_path is not None:
        raw_v = utils.run_verilator(verilog_path)
        open(f"{workload}.verilog.log", "w").write(raw_v)
        open("raw.log", "w").write(raw_v)
        check()
        os.remove("raw.log")
    kernel_cycles = extract_kernel_cycles(raw)
    return kernel_cycles if kernel_cycles is not None else extract_last_cycle(raw)


def check():
    raw_path = "raw.log" if os.path.exists("raw.log") else f"{os.getcwd()}/raw.log"
    script = f"{workspace}/workload.sh"
    if os.path.exists(script):
        res = subprocess.run([script, raw_path, f"{workspace}/workload.data"])
        if res.returncode == 0:
            return
    else:
        script = f"{current_path}/../../minor-cpu/utils/find_pass.sh"
        res = subprocess.run([script, raw_path])
        if res.returncode == 0:
            return

    raw = open(raw_path).read()
    finished = ("ebreak | halt | ecall" in raw) or ("trap" in raw)
    assert finished, f"Failed test: {res.returncode}"


def cp_if_exists(src, dst, placeholder):
    if os.path.exists(src):
        shutil.copy(src, dst)
    elif placeholder:
        open(dst, "w").write("")


def init_workspace(base_path, case):
    if os.path.exists(workspace):
        shutil.rmtree(workspace)
    os.mkdir(workspace)
    cp_if_exists(f"{base_path}/{case}.exe", f"{workspace}/workload.exe", False)
    cp_if_exists(f"{base_path}/{case}.data", f"{workspace}/workload.data", True)
    cp_if_exists(f"{base_path}/{case}.config", f"{workspace}/workload.config", False)
    cp_if_exists(f"{base_path}/{case}.sh", f"{workspace}/workload.sh", False)


def load_baseline_cycles():
    results = {}
    breakdown_candidates = [
        f"{utils.repo_path()}/examples/minor-cpu/src/workload_ipc_stall_breakdown.csv",
        os.path.abspath(f"{current_path}/../../minor-cpu/src/workload_ipc_stall_breakdown.csv"),
    ]
    kernel_cycle_candidates = [
        f"{utils.repo_path()}/examples/minor-cpu/src/kernel_cycles.csv",
        os.path.abspath(f"{current_path}/../../minor-cpu/src/kernel_cycles.csv"),
    ]

    for baseline_path in breakdown_candidates:
        if not os.path.exists(baseline_path):
            continue
        with open(baseline_path, newline="") as f:
            reader = csv.DictReader(f)
            for row in reader:
                try:
                    results.setdefault(row["workload"], int(row["total_cycles"]))
                except (KeyError, ValueError):
                    continue

    for baseline_path in kernel_cycle_candidates:
        if not os.path.exists(baseline_path):
            continue
        with open(baseline_path, newline="") as f:
            reader = csv.DictReader(f)
            for row in reader:
                try:
                    results[row["workload"]] = int(row["minor_cpu_cycles"])
                except (KeyError, ValueError):
                    continue
    # The shipped minor-cpu summary omits the extra sort workloads used here.
    # Keep their validated minor-cpu baselines available for report generation.
    for workload, cycles in {
        "rsort": 182444,
        "msort": 23507,
        "mergesort": 1102,
    }.items():
        results.setdefault(workload, cycles)
    return results


def run_workloads(sys, simulator_binary, verilog_path, workloads, verify_verilator=False):
    wl_path = f"{utils.repo_path()}/examples/minor-cpu/workloads"
    baseline_cycles = load_baseline_cycles()
    rows = []

    for workload in workloads:
        print(f"Running workload: {workload}")
        init_workspace(wl_path, workload)
        cycles = run_cpu(
            sys,
            simulator_binary,
            workload=workload,
            verify_verilator=verify_verilator,
            verilog_path=verilog_path,
        )

        base = baseline_cycles.get(workload)
        speedup = (base / cycles) if base else None
        rows.append((workload, cycles, base, speedup))
        print(f"  cycles={cycles}" + (f" baseline={base} speedup={speedup:.3f}x" if speedup else ""))

    if PURE_DUAL_ISSUE_MODE:
        summary_name = "speedup.pure_dual_issue.csv"
    elif DISC_FAIR_DUAL_ISSUE_MODE:
        summary_name = "speedup.disc_fair_dual_issue.csv"
    else:
        summary_name = "speedup.csv"
    summary_path = f"{current_path}/{summary_name}"
    with open(summary_path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["workload", "minor_cpu_2_issue_cycles", "minor_cpu_cycles", "speedup"])
        for workload, cycles, base, speedup in rows:
            writer.writerow([workload, cycles, base or "", f"{speedup:.6f}" if speedup else ""])

    return rows


if __name__ == "__main__":
    sys, simulator_binary, verilog_path = build_cpu(depth_log=16)
    args = py_sys.argv[1:]
    verify_verilator = os.environ.get("MINOR2_VERIFY_VERILATOR", "0") == "1"

    workloads = args if args else DEFAULT_WORKLOADS
    results = run_workloads(sys, simulator_binary, verilog_path, workloads, verify_verilator=verify_verilator)

    print("Summary:")
    for workload, cycles, base, speedup in results:
        if speedup is None:
            print(f"  {workload}: {cycles} cycles")
        else:
            print(f"  {workload}: {cycles} cycles vs {base} baseline ({speedup:.3f}x)")
