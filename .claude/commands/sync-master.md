---
name: sync-master
description: Synchronize local main/master branch with upstream (or origin) using rebase
---

# Sync Master Command

Synchronize your local main or master branch with the latest changes from the upstream repository.

Invoke the command: `/sync-master`

This command will:
1. Check git status for uncommitted changes
2. Detect the default branch (main or master)
3. Checkout to the detected default branch
4. Detect available remotes (upstream or origin)
5. Pull latest changes using `--rebase`
6. Report success or failure

## Workflow Steps

When this command is invoked, follow these steps:

### Step 1: Check Working Tree Status

Check if there are uncommitted changes:

```bash
git status --porcelain
```

If the output is non-empty, inform the user:

```
Error: Cannot sync - you have uncommitted changes

Please commit or stash your changes before syncing.
```

Stop execution.

### Step 2: Detect Default Branch

Check which default branch exists in the repository:

```bash
git rev-parse --verify main 2>/dev/null || git rev-parse --verify master 2>/dev/null
```

- If `main` exists, use `main`
- Otherwise, if `master` exists, use `master`
- If neither exists, inform the user:

```
Error: Neither 'main' nor 'master' branch found in this repository
```

Stop execution.

### Step 3: Checkout Default Branch

Switch to the detected default branch:

```bash
git checkout <detected-branch>
```

Inform the user:

```
Checking out <detected-branch> branch...
```

### Step 4: Detect Remote

Check which remote to use (prefer upstream, fallback to origin):

```bash
git remote | grep -q "^upstream$"
```

- If `upstream` exists, use `upstream`
- Otherwise, use `origin`

If using fallback, inform the user:

```
upstream remote not found, using origin...
```

### Step 5: Pull with Rebase

Pull the latest changes from the detected remote:

```bash
git pull --rebase <detected-remote> <detected-branch>
```

Inform the user:

```
Pulling latest changes from <detected-remote> with rebase...
```

### Step 6: Report Results

If successful:

```
Successfully synchronized <detected-branch> branch with <detected-remote>/<detected-branch>
```

If rebase conflicts occur, inform the user:

```
Error: Rebase conflict detected

Please resolve conflicts manually:
1. Fix conflicts in the affected files
2. Run: git add <resolved-files>
3. Run: git rebase --continue

Or abort the rebase with: git rebase --abort
```

Stop execution and let the user handle conflicts.

## Error Handling

Following the project's philosophy, assume git tools are available and the repository is properly initialized. Cast errors to users for resolution.

Common error scenarios:
- Uncommitted changes → User must commit or stash
- Branch not found → Inform user
- Rebase conflicts → User resolves manually
- Remote not configured → Git will error naturally
