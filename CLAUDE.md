# Assassyn

Assassyn is a project on next-generation hardware agile design and implementation by providing
a unified interface for both simulation and RTL generation.

## Development Guideline

- To understand the code base, make sure you [read the documents](.cursor/rules/read-the-doc.mdc)
  of both the code itself, and the [high-level design decisions](docs/design/).
- When asked to modify something of the language core module, make sure to
  - [make a plan](.cursor/rules/write-a-plan.mdc) carefully first.
  - maintain the consistency between the [documents and the code carefully](.cursor/rules/document-policy.mdc).
  - [act on this plan](.cursor/rules/act-on-todo.mdc) carefully obeying the rule.
  - When writing code, keep [the standard of high-quality code](.cursor/rules/write-good-code.mdc) in your mind!
  - commit your changes with [a meaningful commit message](.cursor/rules/git-message.mdc).
- When asked to develop a test case or other applications using Assassyn, make sure to read
  the [tutorial](tutorials/) (all the files ends with `_en.qmd` is English) to understand
  how to write an Assassyn program.
- To run anything within this repo:
  - `source setup.sh` to set up the environment. This project depends on several complicated environment variables.
  - `source setup.sh` is even required before running `git commit` so that pre-commit hooks work.
  - Use `source setup.sh && make test-all` to run all tests.
