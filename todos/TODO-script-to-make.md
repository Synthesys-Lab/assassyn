# Goal

Convert the current setup script `init.sh` to `Makefile`.

# Action Items

0. Update `pre-commit` hook to use Make target, `make test-all` instead of sourcing `wrapper.sh`.
1. Create a `Makefile` at the root of this repo as master.
  - Create an `env` target that exports all the environment variable by sourcing `setup.sh`.
  - Create `build-all`, `test-all`, and `clean-all` that the `all` target depend on.
  - Link corresponding targets to `build-all` and `clean-all`:
    - `build-all` depends on: `install-py-package`, `build-verilator`, `build-ramulator2`, `build-wrapper`, `install-circt`
    - `clean-all` depends on: `clean-verilator`, `clean-ramulator2`, `clean-wrapper`, `clean-circt`
2. Convert `scripts/init/py-package.sh` to `scripts/init/py-package.inc` and include to master.
  - Create `install-py-package` target by analyzing the existing `py-package.sh` script.
    - Install Python dependencies from `python/requirements.txt` using pip
    - Use `--user` and `--break-system-packages` flags
  - Make `install-py-package` to ensure it installs successfully.
  - Make `install-py-package` twice to ensure it is idempotent.
     - Also, make sure it does not reinstall when called twice.
  - Stage and commit without verification.
3. Convert `scripts/init/verilator.sh` to `scripts/init/verilator.inc` and include to master.
  - Create `clean-verilator` target by analyzing the existing `verilator.sh` script.
    - Clean build artifacts in `3rd-party/verilator` directory
    - Handle `--no-verilator` flag support
  - Create `build-verilator` target by analyzing the existing `verilator.sh` script.
    - Dependencies: git submodule update, autoconf, configure, make
    - Handle `--no-verilator` flag to skip build
    - Build in `3rd-party/verilator` directory
  - Make `build-verilator` to ensure it builds successfully.
  - Make `build-verilator` twice to ensure it is idempotent.
     - Also, make sure it does not rebuild when called twice.
  - Stage and commit without verification.
4. Convert `scripts/init/wrapper.sh` to `scripts/init/wrapper.inc` and include to master.
  - Create two targets `clean-wrapper` and `clean-ramulator2` by analyzing the existing `wrapper.sh` script.
    - `clean-ramulator2`: Clean build artifacts in `3rd-party/ramulator2/build` and de-apply patch
    - `clean-wrapper`: Clean build artifacts in `tools/c-ramulator2-wrapper/build`
  - `make clean-wrapper clean-ramulator2` to clean both builds
  - Create two targets `build-wrapper` and `build-ramulator2` by analyzing the existing `wrapper.sh` script.
    - **Use elegant Makefile patch management**: Create `3rd-party/ramulator2/.patch-applied` marker file to track patch state
      - `3rd-party/ramulator2/.patch-applied`: Apply patch only if marker doesn't exist, then create marker
      - `build-ramulator2`: Depends on `.patch-applied` marker, then git submodule update, cmake, make
      - **Deprecate complex shell approach**: Replace the complicated `git apply --reverse --check` logic with simple file marker
    - `build-wrapper`: cmake, make in `tools/c-ramulator2-wrapper` (depends on `build-ramulator2`)
  - Make both build targets to make sure they build.
  - Make both build targets twice to make sure they are idempotent.
     - Also, make sure they do not rebuild when twice called.
  - Stage and commit without verification.
5. Convert `scripts/init/circt.sh` to `scripts/init/circt.inc` and include to master.
  - Create `clean-circt` by analyzing [setup.py](../3rd-party/circt/frontends/PyCDE/setup.py).
  - Make clean to clean the build
  - Create target `install-circt` to first install circt from pip, and falls back to build when not available.
  - Make `install-circt` to have the package installed.
     - Also, make sure it does not rebuild when twice called.
  - Stage and commit without verification.
6. Link `test-all` to depend on `build-all`, and test below
   - `pytest -n 8 -x python/unit-tests`.
   - `pytest -n 8 -x python/ci-tests`.
   - Stage and commit.