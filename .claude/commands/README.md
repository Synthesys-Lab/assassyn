# Commands

This directory contains command definitions for Claude Code. Commands are shortcuts that can be invoked to execute specific workflows or skills.

## Purpose

Commands provide a simple interface to invoke complex workflows or skills. Each command is defined in a markdown file with frontmatter metadata.

## Organization

- Each command is defined in its own `.md` file
- Command files include:
  - `name`: The command name (used for invocation)
  - `description`: Brief description of what the command does
  - Instructions on how to use the command and which skills it invokes

## Available Commands

- `git-commit.md`: Invokes the commit-msg skill to create commits with meaningful messages following project standards
- `make-a-plan.md`: Creates comprehensive implementation plans following design-first TDD approach
- `issue-to-impl.md`: Orchestrates full implementation workflow from issue to completion (creates branch, docs, tests, and first milestone)
- `miles2miles.md`: Resumes implementation from the latest milestone on current branch
- `open-issue.md`: Creates GitHub issues from conversation context with proper formatting and tag selection
- `sync-master.md`: Synchronizes local main/master branch with upstream (or origin) using rebase
