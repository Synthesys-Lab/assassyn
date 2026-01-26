import re
from pathlib import Path

from assassyn.frontend import *
from assassyn.test import run_test


ROM_INIT_PATH = Path(__file__).resolve().parent / "resources" / "init_1.hex"


@external
class ExternalRom(ExternalSV):
    """Parameterized ROM external module."""

    addr: WireIn[UInt(8)]
    data: WireOut[UInt(32)]

    __source__: str = "python/ci-tests/resources/rom.sv"
    __module_name__: str = "rom"
    __parameters__ = {
        "ADDR_WIDTH": 8,
        "DATA_WIDTH": 32,
        "INIT_FILE": ROM_INIT_PATH,
    }


class Driver(Module):

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        cnt = RegArray(UInt(8), 1)
        addr = cnt[0]
        cnt[0] = cnt[0] + UInt(8)(1)

        rom = ExternalRom(addr=addr)
        data = rom.data
        log("rom {} {}", addr, data)


def build_system():
    driver = Driver()
    driver.build()


def check(raw: str):
    seen = {}
    for line in raw.splitlines():
        if "rom" not in line:
            continue
        numbers = re.findall(r"-?\d+", line)
        if len(numbers) < 2:
            continue
        addr = int(numbers[-2])
        data = int(numbers[-1])
        seen.setdefault(addr, data)

    for addr in range(4):
        assert seen.get(addr) == addr + 1, (
            f"rom[{addr}] expected {addr + 1}, got {seen.get(addr)}"
        )


def test_external_parameter():
    run_test(
        "external_parameter",
        build_system,
        check,
        sim_threshold=12,
        idle_threshold=12,
    )


if __name__ == "__main__":
    test_external_parameter()
