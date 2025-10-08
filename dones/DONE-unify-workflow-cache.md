# Unify Workflow Cache

[Prior modification](../dones/DONE-fix-workflow-cache.md) partially fixed the issue. However, the Github workflow remains unchanged and lagging.
This TODO aims at fix this.

# Action Items

- As [this modification](../dones/DONE-fix-workflow-cache.md), `wrapper.sh` and `ramulator2.sh` are merged into one, so should [the workflow](../.github/workflows/test.yaml).
  - Unify the wrapper and ramulator build cache, put it right before Pylint.
  - Commit with `--no-verify` as we cannot test it locally.

## Checklist

- [x] **Remove separate ramulator2 cache step from GitHub workflow**
  - [x] Removed `cache-ramulator` step (lines 45-52) that cached only ramulator2 build directory
  - [x] Removed separate ramulator2 build step (lines 53-57) that used `ramulator2.sh`

- [x] **Remove separate wrapper cache step from GitHub workflow**
  - [x] Removed `cache-wrapper` step (lines 71-78) that cached only wrapper build directory
  - [x] Removed separate wrapper build step (lines 79-83) that used `wrapper.sh`

- [x] **Create unified cache step for ramulator2 and wrapper dependencies**
  - [x] Created `cache-ramulator-wrapper` step that caches both ramulator2 and wrapper build directories
  - [x] Unified cache key includes both `ramulator2/.git/HEAD` and `tools/c-ramulator2-wrapper/.git/HEAD` hashes
  - [x] Cache paths include both `3rd-party/ramulator2` and `tools/c-ramulator2-wrapper/build`

- [x] **Update build step to use unified wrapper.sh script**
  - [x] Single build step now uses `scripts/init/wrapper.sh` which handles both ramulator2 and wrapper builds
  - [x] Build step runs only when unified cache misses

- [x] **Commit changes with --no-verify flag**
  - [x] Committed workflow changes with `--no-verify` flag as specified in action items

## Summary

### Checklist Completion
All action items have been successfully completed:

- **Cache Unification**: Removed separate cache steps for ramulator2 and wrapper, replacing them with a unified cache step that covers both dependencies
- **Build Step Consolidation**: Eliminated duplicate build steps and consolidated them into a single step that uses the unified `wrapper.sh` script
- **Cache Key Strategy**: Created a unified cache key that includes both ramulator2 and wrapper git HEAD hashes to ensure proper cache invalidation
- **Workflow Alignment**: GitHub workflow now matches the local unified build process implemented in the prior modification

### Changes Made

**Improvements Made:**
- Unified GitHub workflow cache strategy to match local build process
- Reduced workflow complexity by eliminating duplicate cache and build steps
- Improved cache efficiency by combining related dependencies into single cache entry
- Aligned GitHub workflow with local unified wrapper script approach

**Technical Decisions:**
- Used multi-path cache strategy to cache both ramulator2 and wrapper build directories in single cache entry
- Combined git HEAD hashes in cache key to ensure cache invalidation when either dependency changes
- Maintained cache restore-keys pattern for fallback cache matching
- Used unified wrapper.sh script approach to ensure consistent build process between local and CI environments

The implementation successfully unifies the GitHub workflow cache with the local build process, eliminating the lag between local changes and CI workflow. The workflow now properly reflects the unified build approach implemented in the prior modification.