"""CI test for SRAM masked-write support."""

import re

from assassyn.frontend import *
from assassyn.test import run_test


ADDR0 = 0x12
ADDR1 = 0x34

OPS = [
    {"kind": "write", "addr": ADDR0, "wdata": 0x11223344, "wmask": 0xFFFFFFFF},
    {"kind": "read", "addr": ADDR0, "expected": 0x11223344},
    {"kind": "write", "addr": ADDR0, "wdata": 0xFFFFFFFF, "wmask": 0x00000000},
    {"kind": "read", "addr": ADDR0, "expected": 0x11223344},
    {"kind": "write", "addr": ADDR0, "wdata": 0x000000AA, "wmask": 0x000000FF},
    {"kind": "read", "addr": ADDR0, "expected": 0x112233AA},
    {"kind": "write", "addr": ADDR0, "wdata": 0x0000BB00, "wmask": 0x0000FF00},
    {"kind": "read", "addr": ADDR0, "expected": 0x1122BBAA},
    {"kind": "write", "addr": ADDR0, "wdata": 0x00CC0000, "wmask": 0x00FF0000},
    {"kind": "read", "addr": ADDR0, "expected": 0x11CCBBAA},
    {"kind": "write", "addr": ADDR0, "wdata": 0xDD000000, "wmask": 0xFF000000},
    {"kind": "read", "addr": ADDR0, "expected": 0xDDCCBBAA},
    {"kind": "write", "addr": ADDR0, "wdata": 0x0000EEFF, "wmask": 0x0000FFFF},
    {"kind": "read", "addr": ADDR0, "expected": 0xDDCCEEFF},
    {"kind": "write", "addr": ADDR0, "wdata": 0xA1B20000, "wmask": 0xFFFF0000},
    {"kind": "read", "addr": ADDR0, "expected": 0xA1B2EEFF},
    {"kind": "write", "addr": ADDR1, "wdata": 0x55667788, "wmask": 0xFFFFFFFF},
    {"kind": "read", "addr": ADDR1, "expected": 0x55667788},
    {"kind": "write", "addr": ADDR1, "wdata": 0x00990000, "wmask": 0x00FF0000},
    {"kind": "read", "addr": ADDR1, "expected": 0x55997788},
    {"kind": "write", "addr": ADDR1, "wdata": 0xAA5500CC, "wmask": 0x0F0F00F0},
    {"kind": "read", "addr": ADDR1, "expected": 0x5A9577C8},
    {"kind": "read", "addr": ADDR0, "expected": 0xA1B2EEFF},
]


class ReadObserver(Module):

    def __init__(self):
        super().__init__(
            ports={
                "step": Port(Bits(8)),
                "addr": Port(Bits(9)),
            }
        )

    @module.combinational
    def build(self, rdata: RegArray):
        step, addr = self.pop_all_ports(True)
        log(
            "masked_read step={} addr=0x{:03x} data=0x{:08x}",
            step,
            addr,
            rdata[0].bitcast(Bits(32)),
        )


class Launcher(Module):

    def __init__(self, target):
        super().__init__(ports={})
        self.target = target

    @module.combinational
    def build(self):
        self.target.async_called()


class MaskedDriver(Module):

    def __init__(self, observer):
        super().__init__(ports={})
        self.name = "Driver"
        self.observer = observer

    @module.combinational
    def build(self):
        phase_bits = max(1, (len(OPS) + 1).bit_length())
        phase = RegArray(UInt(phase_bits), 1, initializer=[0])
        state = phase[0]
        next_state = state + UInt(phase_bits)(1)
        (phase & self)[0] <= next_state

        we = Bits(1)(0)
        re = Bits(1)(0)
        addr = Bits(9)(0)
        wdata = Bits(32)(0)
        wmask = Bits(32)(0)

        for idx, op in enumerate(OPS):
            is_step = state == UInt(phase_bits)(idx)
            addr_bits = Bits(9)(op["addr"])
            addr = is_step.select(addr_bits, addr)

            if op["kind"] == "write":
                we = is_step.select(Bits(1)(1), we)
                wdata = is_step.select(Bits(32)(op["wdata"]), wdata)
                wmask = is_step.select(Bits(32)(op["wmask"]), wmask)
            else:
                re = is_step.select(Bits(1)(1), re)
                with Condition(is_step):
                    self.observer.async_called(step=Bits(8)(idx), addr=addr_bits)

        sram = SRAM(32, 512, None)
        sram.build(we, re, addr, wdata, wmask)

        with Condition(state == UInt(phase_bits)(len(OPS))):
            finish()

        return sram


READ_RE = re.compile(
    r"masked_read step=(\d+) addr=0x([0-9a-fA-F]+) data=0x([0-9a-fA-F]+)"
)


def check(raw):
    expected_reads = [
        (idx, op["addr"], op["expected"])
        for idx, op in enumerate(OPS)
        if op["kind"] == "read"
    ]

    actual_reads = []
    for line in raw.splitlines():
        if "[readobserver" not in line.lower():
            continue
        match = READ_RE.search(line)
        assert match is not None, f"Unexpected ReadObserver log line: {line}"
        actual_reads.append(
            (
                int(match.group(1)),
                int(match.group(2), 16),
                int(match.group(3), 16),
            )
        )

    assert actual_reads == expected_reads, (
        f"Masked SRAM reads mismatch.\n"
        f"expected={expected_reads}\n"
        f"actual={actual_reads}"
    )


def test_sram_masked_write():
    def top():
        observer = ReadObserver()
        driver = MaskedDriver(observer)
        sram = driver.build()
        launcher = Launcher(driver)
        launcher.build()
        observer.build(sram.dout)

    run_test("sram_masked", top, check, sim_threshold=200, idle_threshold=200)


if __name__ == "__main__":
    test_sram_masked_write()
