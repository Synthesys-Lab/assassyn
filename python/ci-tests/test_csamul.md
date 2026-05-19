# Legacy CSA Multiplier Test Entrypoint

`test_csamul.py` keeps the historical `test_multiplier` pytest name alive
while delegating to the deterministic CSA multiplier coverage in
[`test_csa_multiplier.py`](./test_csa_multiplier.py).

## Interface Exposed

- `test_multiplier`: Imported alias for `test_csa_multiplier()`. Pytest and
  direct script execution both use this name for compatibility with older
  workflows.

## Internal Helpers

This file defines no local helper functions. The multiplier system builder,
raw-log checker, vectors, and regex parsing live in `test_csa_multiplier.py` so
the compatibility entrypoint cannot drift from the main coverage.

## Data Structures

No local data structures are defined here. All test vectors and parser state
are owned by `test_csa_multiplier.py`.
