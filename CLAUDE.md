# Assassyn

Assassyn is a project on next-generation hardware agile design and implementation by providing a unified interface for both simulation and RTL generation.

## Development Guideline

- Use `source setup.sh` to set up the environment before running any scripts.
  - Even for `git commit`, because of pre-commit hooks.
- When it comes testing, do `pytest -n 8 -x python/ci-tests` to speedup and stop at first failure.
- `pylint` is for `python/assassyn` only, and the rc file is `python/assassyn/.pylintrc`.
  - If some refactoring is made, first run `python/ci-tests/test_driver.py` as a sanity check.
- With regard to `TODO-something.md`:
  - When told to review a `TODO-something.md` follow the [guideline of a todo list](./docs/developer/todo.md).
  - When told to act on a `TODO-something.md` [development workflow](./docs/developer/flow-on-todo.md) to work on it accordingly. Action items inside are the only case you are allowed to bypass precommit checks using `--no-verify`.
- For most general cases, after finishing a feature, stage and commit it with a message discribed in [git-message.md](./docs/developer/git-message.md).
