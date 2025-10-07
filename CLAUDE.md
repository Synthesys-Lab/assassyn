- Use `source setup.sh` to set up the environment before running any scripts.
  - Even for `git commit`, because of pre-commit hooks.
- When it comes testing, do `pytest -n 8 -x python/ci-tests` to speedup and stop at first failure.
- `pylint` is for `python/assassyn` only, and the rc file is `python/assassyn/.pylintrc`.
  - If some refactoring is made, first run `python/ci-tests/test_driver.py` as a sanity check.
- When told act on `TODO-something.md`, refer to the flow in
  [development workflow](./docs/developer/flow-on-todo.md) to work on it accordingly.
  - This clarifies some cases you can use `git commit --no-verify`.
- For most general cases, after finishing a feature, stage and commit it with a message
  discribed in [github.md](./docs/developer/github.md).
