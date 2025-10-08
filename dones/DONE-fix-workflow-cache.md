# Fix Git Cache Hit for Unify Library Path

This is a follow up to [unifying lib path](../dones/DONE-unify-lib-path.md). Two lib path files `.cwrapper-lib-path` and  `.ramulator2-lib-path` are not part of [wrapper](../tools/c-ramulator2-wrapper/), which does not trigger a rebuild, which shall be solved. Otherwise, we do not have these two files at all to properly test.

## Action Items

1. Make `.cwrapper-lib-path` and `.ramulator2-lib-path` generated in wrapper build folder.
2. Refer to [unifying lib path](../dones/DONE-unify-lib-path.md) find all the files referring to these files use `$ASSASSYN_HOME/../tools/c-ramulator2-wrapper/build`
3. Currently, we have two separate scripts for [wrapper](../scripts/init/wrapper.sh) and [ramulator2](../scripts/init/ramulator2.sh). Merge these two into one, as wrapper always depends on this.
4. Fix the `pre-commit` hook that runs `ramulator2.sh` by running `wrapper.sh`.

## Checklist

- [x] **CMake Path Generation Fix**
  - [x] Modified `tools/c-ramulator2-wrapper/CMakeLists.txt` to generate `.cwrapper-lib-path` and `.ramulator2-lib-path` in wrapper build directory instead of python directory
  - [x] Updated template files to include platform-specific library extensions using `@CMAKE_SHARED_LIBRARY_SUFFIX@`

- [x] **File Reference Updates**
  - [x] Updated `python/assassyn/ramulator2/ramulator2.py` to read path files from wrapper build directory
  - [x] Updated `tools/rust-sim-runtime/src/ramulator2.rs` to read path files from wrapper build directory
  - [x] Fixed `load_shared_library()` function in Python to handle paths that already have extensions
  - [x] Fixed `load_library!` macro in Rust to handle paths that already have extensions

- [x] **Script Consolidation**
  - [x] Merged `scripts/init/wrapper.sh` and `scripts/init/ramulator2.sh` into unified wrapper script
  - [x] Updated `scripts/pre-commit` to use unified wrapper script instead of separate ramulator2 script

- [x] **Testing and Verification**
  - [x] Verified wrapper build generates path files in correct location
  - [x] Verified Python wrapper can load libraries from new path location
  - [x] Verified Rust wrapper can load libraries from new path location
  - [x] Ran comprehensive test driver to ensure simulator and Verilog generation work correctly

## Summary

### Checklist Completion
All action items have been successfully completed:

- **CMake Path Generation Fix**: Modified CMakeLists.txt to generate library path files in the wrapper build directory (`tools/c-ramulator2-wrapper/build/`) instead of the Python directory, ensuring proper cache invalidation when wrapper changes
- **File Reference Updates**: Updated all files that reference the library path files to use the new location in the wrapper build directory, including Python wrapper and Rust runtime
- **Script Consolidation**: Merged the separate wrapper and ramulator2 build scripts into a unified wrapper script since wrapper always depends on ramulator2
- **Testing and Verification**: Verified that all components work correctly with the new path setup through comprehensive testing

### Changes Made

**New Features Added:**
- Library path files now generated in wrapper build directory for proper Git cache invalidation
- Unified build script that handles both ramulator2 and wrapper dependencies
- Enhanced library loading functions that handle paths with existing extensions

**Improvements Made:**
- Fixed Git cache hit issue by ensuring library path files are part of wrapper build process
- Eliminated duplicate build scripts by consolidating ramulator2 and wrapper builds
- Improved library loading robustness by handling paths with existing extensions
- Updated pre-commit hook to use unified build process

**Technical Decisions:**
- Used CMake's `@CMAKE_SHARED_LIBRARY_SUFFIX@` variable to generate platform-specific library extensions in template files
- Modified library loading functions to check if paths already have extensions before adding them
- Maintained backward compatibility by keeping the same public API while changing internal path resolution
- Used unified build script approach to ensure proper dependency ordering between ramulator2 and wrapper

The implementation successfully fixes the Git cache hit issue by ensuring that library path files are generated as part of the wrapper build process, which will trigger rebuilds when the wrapper changes. All tests pass and the system maintains full functionality.