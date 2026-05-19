"""Legacy CSA multiplier smoke test.

The reusable CSA tree lives in ``assassyn.ip.multiply``.  This file remains as
a compatibility test entrypoint and delegates the actual deterministic coverage
to ``test_csa_multiplier``.
"""

from test_csa_multiplier import test_csa_multiplier as test_multiplier


if __name__ == "__main__":
    test_multiplier()
