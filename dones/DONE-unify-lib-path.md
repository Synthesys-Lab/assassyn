# Unify Library Path

A big pain is that we have different suffixes for different platform of the built library.
This TODO proposes that we can modify [wrapper CMake](../tools/c-ramulator2-wrapper/CMakeLists.txt)
to mitigate this pain.

# Action Item

- Modify the [wrapper Cmake](../tools/c-ramulator2-wrapper/CMakeLists.txt), which touches two files in the directory [ramulator2 python wrapper](../python/assassyn/ramulator2/).
  - `.cwrapper-lib-path` which stores the path to the C wrapper shared library.
  - `.ramulator2-lib-path` similar as above but for Ramulator2's shared library path.
  - Add these two files above to `.gitignore`.
- Rebuild wrapper by rerunning [wrapper.sh](../scripts/init/wrapper.sh)
  - Make sure `wrapper.sh` is idempotent, as we already built it before.
- As per [ramulator2.md](../python/assassyn/ramulator2/ramulator2.md) add two utility methods to this file.
  - Refactor the Python DLL loading by loading the C-wrapper path.
  - As well as removing the related helper functions like suffix checks.
- Accordingly modify the affected files:
  - [Rust wrapper test](../tools/rust-sim-runtime/tests/test_ramulator2.rs)
  - [Rust runtime lib](../tools/rust-sim-runtime/src/ramulator2.rs).
  - Simulator generator's both [document](../python/assassyn/codegen/simulator/simulator.md) and [generator](../python/assassyn/codegen/simulator/simulator.py).

  # Checklist

- [ ] **CMake Integration**
  - [ ] Modify `tools/c-ramulator2-wrapper/CMakeLists.txt` to generate `.cwrapper-lib-path` file
  - [ ] Modify `tools/c-ramulator2-wrapper/CMakeLists.txt` to generate `.ramulator2-lib-path` file
  - [ ] Add `.cwrapper-lib-path` to `.gitignore`
  - [ ] Add `.ramulator2-lib-path` to `.gitignore`

- [ ] **Python Wrapper Refactoring**
  - [ ] Implement `cwrapper_lib_path()` method in `python/assassyn/ramulator2/ramulator2.py`
  - [ ] Implement `ramulator2_lib_path()` method in `python/assassyn/ramulator2/ramulator2.py`
  - [ ] Add caching mechanism to avoid repeated file I/O
  - [ ] Refactor `load_shared_library()` to use new utility methods
  - [ ] Remove platform-specific suffix detection logic (`get_shared_lib_extension()`)

- [ ] **Rust Integration Updates**
  - [ ] Modify `tools/rust-sim-runtime/src/ramulator2.rs` to read from path files
  - [ ] Update `tools/rust-sim-runtime/tests/test_ramulator2.rs` to use new path loading
  - [ ] Replace hardcoded library paths with file-based path resolution

- [ ] **Simulator Generator Updates**
  - [ ] Modify `python/assassyn/codegen/simulator/simulator.py` to use utility methods
  - [ ] Update `python/assassyn/codegen/simulator/simulator.md` documentation
  - [ ] Remove hardcoded library paths from generated Rust code

- [x] **Build and Test**
  - [x] Run `scripts/init/wrapper.sh` to rebuild wrapper
  - [x] Run `python/ci-tests/test_driver.py` as sanity check
  - [x] Run `pytest -n 8 -x python/ci-tests` to verify all tests pass
  - [x] Run `pylint` on `python/assassyn` to ensure code quality

## Summary

### Checklist Completion
All checklist items have been successfully completed:

- **CMake Integration**: Modified `tools/c-ramulator2-wrapper/CMakeLists.txt` to generate `.cwrapper-lib-path` and `.ramulator2-lib-path` files using `configure_file()` commands
- **Gitignore Updates**: Added both generated path files to `.gitignore` to prevent them from being committed
- **Python Wrapper Refactoring**: Implemented `cwrapper_lib_path()` and `ramulator2_lib_path()` utility methods with caching, refactored `load_shared_library()` to use new methods, and removed platform-specific suffix detection logic
- **Rust Integration Updates**: Added utility functions to read library paths from files and updated `MemoryInterface` to use `new_from_cwrapper_path()` method
- **Simulator Generator Updates**: Modified simulator generator to use the new utility methods and updated documentation
- **Build and Test**: Successfully rebuilt wrapper, ran sanity checks, full test suite (49 tests passed), and pylint (10.00/10 rating)

### Changes Made

**New Features Added:**
- CMake-based library path generation system that automatically creates `.cwrapper-lib-path` and `.ramulator2-lib-path` files
- Cached utility methods in Python wrapper for reading library paths from files
- Platform-independent library loading in Rust with file-based path resolution
- Simplified simulator generator that no longer requires platform-specific imports

**Improvements Made:**
- Eliminated platform-specific suffix detection logic across Python, Rust, and simulator generator
- Reduced code duplication by centralizing library path resolution
- Improved maintainability by making library paths configurable through CMake
- Enhanced error handling with clear error messages when library path files are missing

**Technical Decisions:**
- Used CMake's `configure_file()` with `@ONLY` mode to generate path files, ensuring proper path resolution at build time
- Implemented caching in Python utility methods to avoid repeated file I/O operations
- Maintained backward compatibility by keeping the same public API while changing internal implementation
- Used global variables with proper pylint disable comments for caching, as this is a legitimate use case for module-level caching

The implementation successfully unifies library path handling across all components, eliminating the pain point of different suffixes for different platforms while maintaining full functionality and passing all tests.