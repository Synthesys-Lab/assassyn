---
name: git-commit
description: Create a git commit with meaningful commit messages following project standards
---

# Git Commit Command

Execute the commit-msg skill to commit staged changes with meaningful commit messages.

Invoke the skill: /git-commit

This command will:
1. Analyze staged changes using `git diff --staged`
2. Review the commit message tag standards in `docs/git-msg-tags.md`
3. Create an appropriate commit message following the format defined in the commit-msg skill
4. Execute the commit without bypassing pre-commit hooks

