---
alwaysApply: false
description: How to act on a given TODO document
glob:
---

# Developer Flow

This document outlines the flow both a human and an agentic AI developer shall follow to
add new features, fix bugs, and improve the codebase, when a `TODO-xxx.md` as described
[here](./resolve-a-todo.mdc) is given.

## 1. Understand the Goal

Look at the `Goal` section in the `TODO-xxx.md` file. Make sure you understand what needs to be done.

## 2. Analyze the Current State

It is supposed that all the design documents are updated to the desired state, but the
code is still lagging when `TODO-xxx.md` is received. Review the design documents and incorperate
the action items into your understanding of the codebase.

## 3. Act on Action Items

Follow the action items in the `TODO-xxx.md` file step by step. Faithfully implement the changes
and improvements as described in the action items. Faithfully use `git commit` with or without
`--no-verify` flag as described in the action items.

## 4. Checklist & Summary

After all action items are down, a summary checklist should be appended after the `TODO-xxx.md` file.
After that, move `TODO-xxx.md` to `dones/DONE-xxx.md` where `xxx` is the same as `TODO-xxx`.
  - If `dones` folder does not exist, create it.

The summary should include two parts:
1. Check all the checklist items in the `TODO-xxx.md` file are done.
2. Summarize the changes made in the codebase, including:
   - New features added
   - Bugs fixed
   - Improvements made
   - Any other relevant information
   - If an interface refactor happens more than 3 times, present a re-presentative simple before-after code snippet.
      - No need to present code snippets in other cases.
3. Summarize the non-obvious technical decisions made during the implementation. This includes but is not limited to:
   - For example, if it is a short-term hack, explain this hack and suggest a fundamental solution.
   - If a workaround for a bug in test case before this TODO, explain it and suggest a fundamental solution.
   - If a test case is skipped, explain why and suggest a plan to unskip it.
   - If an external dependency does not fulfill our need, explain why and suggest a plan to replace it.

After makine all these changes above, stage and commit all the changes as per our git commit message guideline.