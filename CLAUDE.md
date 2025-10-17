# Assassyn

Assassyn is a project on next-generation hardware agile design and implementation by providing a unified interface for both simulation and RTL generation.

## Development Guideline

- Use `setup.sh` to set up the environment before running any scripts.
  - Run it as `source setup.sh` when you need the environment in the current shell (even before `git commit` so pre-commit hooks work).
- When staging and committing:
  - Follow the rule in `docs/developer/git-message.md`
  - This rule also applies to pull requests.
  - Claude Code, DO NOT claim your co-authorship in the commit message.
- The high-level design can be found in `docs/design/`.
- When it comes testing, use `make test-all`.
- `pylint` is for `python/assassyn` only, and the rc file is `python/assassyn/.pylintrc`.
  - If some refactoring is made, first run `python/ci-tests/test_driver.py` as a sanity check.
