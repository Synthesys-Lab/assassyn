"""Deterministic Rust-simulator coverage for the shared CSA multiplier IP."""

import re

from assassyn.frontend import *
from assassyn.ip.multiply import multiply
from assassyn.test import run_test


VECTORS = (
    (0x00000000, 0x00000000),
    (0x00000001, 0x00000000),
    (0x00000000, 0x00000001),
    (0x00000001, 0x00000001),
    (0x00000003, 0x00000007),
    (0x0000FFFF, 0x00010001),
    (0x12345678, 0x00000009),
    (0x80000000, 0x00000002),
    (0xFFFFFFFF, 0x00000002),
    (0xFFFFFFFF, 0xFFFFFFFF),
)

RESULT_RE = re.compile(
    r"CsaMultiplierResult: tag=(?P<tag>\d+) "
    r"a=(?P<a>\d+) b=(?P<b>\d+) product=(?P<product>\d+)"
)


def _select_vector_value(selector: Value, values: tuple[int, ...]) -> Value:
    """Select one 32-bit constant from ``values`` using an Assassyn case tree."""
    cases = {UInt(32)(index): UInt(32)(value) for index, value in enumerate(values)}
    cases[None] = UInt(32)(0)
    return selector.case(cases)


class Driver(Module):
    """Cycle through fixed operands and feed the shared multiplier pipeline."""

    def __init__(self):
        super().__init__(ports={})

    @module.combinational
    def build(self):
        cycle = RegArray(UInt(32), 1)
        cycle[0] = cycle[0] + UInt(32)(1)

        selector = cycle[0] % UInt(32)(len(VECTORS))
        a = _select_vector_value(selector, tuple(vector[0] for vector in VECTORS))
        b = _select_vector_value(selector, tuple(vector[1] for vector in VECTORS))
        product = multiply(a, b, cycle[0], debug=True)

        log("CsaMultiplierObserved: cycle={} product={}", cycle[0], product)


def build_system():
    """Build the deterministic multiplier test system."""
    driver = Driver()
    driver.build()


def check_raw(raw: str):
    """Check every multiplier result log and require all fixed vectors."""
    seen = set()
    checked = 0

    for line in raw.splitlines():
        match = RESULT_RE.search(line)
        if not match:
            continue

        a = int(match.group("a"))
        b = int(match.group("b"))
        product = int(match.group("product"))
        expected = (a * b) & ((1 << 64) - 1)
        assert product == expected, (
            f"bad CSA multiplier product for {a:#x} * {b:#x}: "
            f"got {product:#x}, expected {expected:#x}"
        )
        seen.add((a, b))
        checked += 1

    assert checked >= len(VECTORS), f"checked only {checked} multiplier results"
    missing = set(VECTORS) - seen
    assert not missing, f"missing multiplier vectors: {sorted(missing)}"


def test_csa_multiplier():
    """Run deterministic CSA multiplier coverage in the Rust simulator."""
    run_test(
        "csa_multiplier",
        build_system,
        check_raw,
        sim_threshold=80,
        idle_threshold=120,
        verilog=False,
    )


if __name__ == "__main__":
    test_csa_multiplier()
