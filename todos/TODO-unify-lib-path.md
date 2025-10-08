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

- [ ] **Build and Test**
  - [ ] Run `scripts/init/wrapper.sh` to rebuild wrapper
  - [ ] Run `python/ci-tests/test_driver.py` as sanity check
  - [ ] Run `pytest -n 8 -x python/ci-tests` to verify all tests pass
  - [ ] Run `pylint` on `python/assassyn` to ensure code quality