- When staging and committing:
  - Follow the rule in ./docs/developer/github.md.
  - This rule also applies to pull requests.
  - Claude Code, DO NOT claim your co-authorship in the commit message.
- When it comes testing, do `pytest -n 8 python/unit-tests` to speedup.
- `pylint` is for `python/assassyn` only, and the rc file is `python/assassyn/.pylintrc`.
- When it comes to "based on my modifications to something.md", use `git diff`
  to read the deltas.
