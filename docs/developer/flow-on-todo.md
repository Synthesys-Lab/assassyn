# Developer Flow

This document outlines the flow both a human and an agentic AI developer shall follow to
add new features, fix bugs, and improve the codebase, when a `TODO-xxx.md` as described
[here](./todo.md) is given.

## 1. Understand the Goal

Look at the `Goal` section in the `TODO-xxx.md` file. Make sure you understand what needs to be done.

## 2. Analyze the Current State

It is supposed that all the design documents are updated to the desired state, but the
code is still lagging when `TODO-xxx.md` is received. Review the design documents and incorperate
the action items into your understanding of the codebase.

## 3. Act on Action Items

Follow the action items in the `TODO-xxx.md` file step by step. For each action item,
it shall be two sub-steps:
1. Create new tests as per the new design.
   - For this step, you may need to carefully read the design documents and the existing
     tests, as well as the developer's test document (if exists) to understand how to
     write the tests.
   - After writing the tests, run them to make sure: 1) they do not have syntax errors;
     2) they fail as expected.
   - Commit the new tests with a message like "Add new test for xxx feature/bug fix".
     Remember to use `--no-verify` to skip pre-commit hooks. Currently, this is the
     ONLY STEP allowed to skip pre-commit hooks.
2. Implement the changes as per the new design.
   - For this step, you may need to carefully read the design documents and the existing
     code to understand how to implement the changes.
   - After implementing the changes, run all the tests to make sure they pass.
   - Commit the changes with a message like "Implement xxx feature/bug fix in `<related files>`".
   - Commit should have pre-commit hooks triggerred hooks to ensure code quality in both linting and testing.
   - If the commit fails due to pre-commit hooks, fix the issues and try again.
3. Repeat the above two sub-steps for each action item in the `TODO-xxx.md` file.

## 4. Checklist & Summary

After all action items are down, a summary checklist should be listed as per the `TODO-xxx.md` file.
The summary should include two parts:
1. Check all the checklist items in the `TODO-xxx.md` file are done.
2. Summarize the changes made in the codebase, including:
   - New features added
   - Bugs fixed
   - Improvements made
   - Any other relevant information
   - If it is a immersive refactor, present a simple before-after code snippet.
3. Summarize the non-obvious technical decisions made during the implementation. This includes but is not limited to:
   - For example, if it is a short-term hack, explain this hack and suggest a fundamental solution.
   - If a workaround for a bug in test case before this TODO, explain it and suggest a fundamental solution.
   - If a test case is skipped, explain why and suggest a plan to unskip it.
   - If an external dependency does not fulfill our need, explain why and suggest a plan to replace it.

Dump all the summary above to `dones/DONE-xxx.md` file, where `xxx` is the same as in `TODO-xxx.md`.
  - If `dones` folder does not exist, create it.
