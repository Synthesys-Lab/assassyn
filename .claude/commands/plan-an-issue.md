---
name: plan-an-issue
description: Create GitHub [plan] issues from implementation plans with proper formatting
argument-hint: [file-path]
---

# Plan an Issue Command

Create GitHub issues tagged as `[plan][tag]` from implementation plans.
This command creates issues that include detailed implementation plans as the "Proposed Solution" section.

Invoke the command: `/plan-an-issue [file-path]`

If arguments are provided via $ARGUMENTS, the skill will use them as input (typically a path to the plan file).
Otherwise, the command will look for a plan in the conversation context.

## What This Command Does

This command creates a [plan] issue on GitHub. It will:
1. Locate the implementation plan
    - If a file path is provided via $ARGUMENTS, read that plan file
    - If no file is given, look for a plan in the conversation context (created by `make-a-plan` skill)
2. Review tag standards in `docs/git-msg-tags.md`
3. Determine the appropriate tag for the issue (e.g., `[plan][feature]`, `[plan][refactor]`)
4. Draft the issue with proper formatting, using the plan as the "Proposed Solution" section
5. Confirm with the user before creating via `gh issue create`

## Issue Format

[plan] issues follow this structure:

```
Title: [plan][tag] Brief description

## Problem Statement
Description of what needs to be done and why

## Proposed Solution
[The implementation plan goes here]

## Test Strategy
How the changes will be tested
```

## Workflow with Planning

The typical workflow is:

1. First run: `/make-a-plan` to create the implementation plan
2. Review and approve the plan
3. Then run: `/plan-an-issue` to create the GitHub issue with the plan

Alternatively, if you already have a plan file:

```
/plan-an-issue path/to/plan.md
```

## When to Use This Command

Use `/plan-an-issue` for:
- New features requiring implementation details
- Refactoring tasks with multiple file changes
- Improvements with specific implementation approach
- Any issue that needs a `[plan][tag]` prefix

The plan should include:
- Files to be created or modified
- Key implementation decisions
- Architectural considerations
- Step-by-step implementation approach
