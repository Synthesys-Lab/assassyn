"""Legacy CSA multiplier smoke test.

The reusable CSA tree lives in ``assassyn.ip.multiply``.  This file remains as
a compatibility test entrypoint and delegates the actual deterministic coverage
to ``test_csa_multiplier``.
"""

from test_csa_multiplier import build_system, check_raw

from assassyn.test import run_test


def test_multiplier():
    """Run the shared multiplier IP through the Rust simulator."""
    run_test("csamul_compat", build_system, check_raw, verilog=False)


if __name__ == "__main__":
    test_multiplier()
