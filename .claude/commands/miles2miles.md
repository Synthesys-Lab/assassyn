---
name: miles2miles
description: Resume implementation from the latest milestone on current branch
argument-hint: [milestone-file]
---

# Miles2Miles Command

Resume implementation from a milestone checkpoint.

## Invocation

```
/miles2miles [milestone-file]
```

**Arguments:**
- `milestone-file` (optional): Path to specific milestone file (e.g., `.milestones/issue-42-milestone-2.md`). If not provided, auto-detects latest milestone on current branch.

## Inputs

**From arguments:**
- Milestone file path (optional): If provided, use this specific milestone; otherwise auto-detect

**From git (if no argument provided):**
- Current branch name (must match `issue-{N}-{brief-title}` pattern)
- Issue number extracted from branch name

**From filesystem (if no argument provided):**
- Latest milestone file: `.milestones/issue-{N}-milestone-{M}.md` (highest M)

**From milestone document:**
- Header metadata: branch, created datetime, LOC implemented, test status
- Work Remaining section: incomplete implementation steps
- Next File Changes section: files to modify with LOC estimates
- Test Status section: passed tests, not passed tests

## Outputs

**Files created/modified:**
- Implementation files (from milestone's "Next File Changes" and "Work Remaining")
- Optional: Additional milestone document `.milestones/issue-{N}-milestone-{M+1}.md`

**Git commits:**
- Optional: Milestone {M+1} commit (incremental progress, X/N tests passed)
- Optional: Delivery commit (all tests passed)

**Terminal output:**
- Success: "Implementation complete: {LOC} LOC, {N}/{N} tests passed. Next Step: Review and create PR."
- Or: "Milestone {M+1} created at {LOC} LOC ({passed}/{total} tests passed). Next Step: /miles2miles .milestones/issue-{N}-milestone-{M+1}.md"

## Skill Integration

### Step 1: Determine Milestone File

**If `$ARGUMENTS` provided:**
- Use argument as milestone file path
- Validate file exists: `[ -f "$ARGUMENTS" ]`
- Extract issue number from filename: `issue-{N}-milestone-{M}.md`

**If no argument provided:**
- Check current branch:
  ```bash
  git branch --show-current
  ```
- Validate branch name matches pattern `issue-{N}-{brief-title}`
- Extract issue number `{N}` from branch name
- Search for latest milestone:
  ```bash
  ls -1 .milestones/issue-{N}-milestone-*.md 2>/dev/null | sort -V | tail -n 1
  ```

**Error handling:**
- Argument provided but file not found:
  ```
  Error: Milestone file not found: {argument}

  Please provide a valid milestone file path.
  ```
  Stop execution.

- No argument and not on development branch:
  ```
  Error: Not on a development branch.

  Current branch: {branch-name}

  You must be on a development branch (issue-{N}-{brief-title}) or provide a milestone file path.
  ```
  Stop execution.

- No argument and no milestones found:
  ```
  Error: No milestone files found for issue #{N}.

  Searched for: .milestones/issue-{N}-milestone-*.md

  This branch does not have any milestones yet.
  ```
  Stop execution.

### Step 2: Load Milestone Context

**Read milestone file:**
```bash
cat .milestones/issue-{N}-milestone-{M}.md
```

**Parse and extract:**

1. **From header (metadata):**
   - Branch name
   - Created datetime
   - LOC implemented: `~{loc} lines`
   - Test status: `{passed}/{total} tests passed`

2. **From "Work Remaining" section:**
   - List of incomplete implementation steps
   - Files to modify with descriptions
   - LOC estimates per step

3. **From "Next File Changes" section:**
   - Specific files to modify next
   - Estimated LOC per file
   - Total estimated for next milestone

4. **From "Test Status" section:**
   - Passed tests: list with names/descriptions
   - Not passed tests: list with names/descriptions and error details

**Error handling:**
- Milestone file corrupted or missing required sections:
  ```
  Error: Milestone file corrupted or improperly formatted.

  File: .milestones/issue-{N}-milestone-{M}.md

  Missing required sections: [list]
  ```
  Stop execution, show file path for manual inspection.

### Step 3: Display Milestone Summary

**Output to user:**
```
Resuming from Milestone {M} for Issue #{N}

Branch: issue-{N}-{brief-title}
Created: {datetime}
LOC implemented: ~{loc}
Test status: {passed}/{total} tests passed

Work remaining:
{work-remaining-summary}

Estimated next milestone: ~{estimated-loc} LOC

Not passed tests ({failed-count}):
{list-of-failed-tests}

Starting implementation...
```

### Step 4: Invoke Milestone Skill

**Invoke:** `milestone` skill
**Input:**
- Branch context: current branch (issue-{N}-*)
- Plan reference: latest milestone document (`.milestones/issue-{N}-milestone-{M}.md`)
- Starting LOC count: {loc} (from milestone header)
- Current test status: {passed}/{total} tests passed

**Milestone skill behavior:**
1. Reads "Work Remaining" and "Next File Changes" from milestone
2. Implements code in chunks (100-200 LOC per chunk)
3. Runs tests after each chunk
4. Tracks cumulative LOC from starting count
5. Stops when:
   - **Cumulative LOC ≥ starting + 800 AND tests incomplete** → Create Milestone {M+1}
   - **All tests pass** → Signal completion

**Handle milestone skill output:**

**Output A: Next milestone created**
```
Milestone {M+1} created at {new-LOC} LOC ({passed}/{total} tests passed).

Work remaining: ~{estimated} LOC
Tests failing: {list}

Next Step: /miles2miles .milestones/issue-{N}-milestone-{M+1}.md
```
Command outputs the new milestone file path for next invocation.

**Output B: All tests pass (completion)**
```
All tests passed ({total}/{total})!

Implementation complete:
- Total LOC: ~{LOC}
- All {total} tests passing

Next Step: Review changes and create PR with /open-pr
```
Command completes successfully.

**Output C: Critical error**
```
Critical errors detected. Milestone {M+1} created with error notes.

Errors:
- {error descriptions}

Next Step: Fix errors, then /miles2miles .milestones/issue-{N}-milestone-{M+1}.md
```
Command outputs the milestone file with error notes.

## Error Handling

### Milestone File Not Found (with argument)

Argument provided but file doesn't exist.

**Response:**
```
Error: Milestone file not found: {argument}

Please provide a valid milestone file path.
```
Stop execution.

### Not on Development Branch (no argument)

No argument provided and current branch name doesn't match `issue-{N}-*` pattern.

**Response:**
```
Error: Not on a development branch.

Current branch: {branch-name}

You must be on a development branch (issue-{N}-{brief-title}) or provide a milestone file path.
```
Stop execution.

### No Milestones Found (no argument)

No argument provided and no `.milestones/issue-{N}-milestone-*.md` files exist.

**Response:**
```
Error: No milestone files found for issue #{N}.

Searched in: .milestones/

This could mean:
1. You haven't started implementation yet
2. You're on the wrong branch
3. The .milestones/ directory doesn't exist

Provide a milestone file path or use /issue-to-impl to start implementation.
```
Stop execution.

### Milestone File Corrupted

Milestone file exists but cannot be parsed (missing required sections).

**Response:**
```
Error: Milestone file corrupted or improperly formatted.

File: .milestones/issue-{N}-milestone-{M}.md

The milestone document is missing required sections:
- [Missing sections list]
```
Stop execution. Display file path for manual inspection.

### Uncommitted Changes

Working directory has uncommitted changes.

```bash
git status --short
# Output: M file1.py, ?? file2.py
```

**Response:**
```
Warning: You have uncommitted changes.

Modified files:
{list-of-modified-files}

Please commit or stash your changes before resuming with /miles2miles.

The milestone skill needs a clean working directory to track LOC accurately.
```
Ask user to handle uncommitted changes before proceeding.
