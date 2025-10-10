# Polish Makefile

This TODO is a follow up of [prior TODO](../dones/DONE-script-to-make.md).
As all the [makefile.inc in the folder](../scripts/init/) were converted from `.sh` files, so `$?` checks were faithfully converted, which can be omitted as Makefile will automatically fail.

# Action Items

1. Simplify all the [makefile.inc in the folder](../scripts/init/) by eliminating the `$?` checks.
   - All the `if [ $? -eq 0 ];` or equivalent shall be removed, as Makefile will auto fail.
2. Move all the patches to [makefile.inc folder](../scripts/init/), and fix their usages in related makefile, by creating a `patches` folder inside.
   - Create `scripts/init/patches/` directory
   - Move and rename patches as specified below
   - Update `wrapper.inc` to reference new patch locations (e.g., `../../scripts/init/patches/ramulator2.patch`)
   - Test that patch application and removal still work correctly
   - Rename [verilator patch](../scripts/5222-gnu-20.patch) to `verilator.patch`.
     - Create a simple doc associated, `verilator-patch.md`, saying fix GNU standard compatibility.
   - Rename [circt patch](../scripts/circt-disable-esi.patch) to `circt.patch`.
     - Create a simple doc associated, `circt-patch.md`, saying disable esi to minimize dependences.
   - Rename [ramulator2 patch](../scripts/ramulator2-template.patch) to `ramulator2.patch`.
     - Also remember to move [the doc](../scripts/ramulator2-patch.md).
3. Run `make clean-all` and `make test-all` to make sure everything works.
   - Test individual components: `make build-verilator`, `make build-ramulator2`, etc.
   - Verify idempotency by running build targets twice
4. Stage and commit!
    - DO NOT bypass!

# Summary

## Action Items Completed
- [x] Simplify all makefile.inc files by eliminating $? checks
- [x] Create scripts/init/patches/ directory
- [x] Move and rename patches to patches directory with documentation
- [x] Update wrapper.inc to reference new patch locations
- [x] Run make clean-all and make test-all to verify everything works
- [x] Stage and commit all changes

## Changes Made

### New Features Added
- Centralized patch management system in `scripts/init/patches/` directory
- Documentation files for verilator and circt patches explaining their purpose

### Improvements Made
- Simplified makefile.inc files by removing unnecessary `$?` checks
- Better organization of patches with descriptive names and documentation
- Improved maintainability of build system

### Files Modified
- `scripts/init/wrapper.inc`: Removed `$?` checks and updated patch paths
- `scripts/init/verilator.inc`: Removed `$?` checks
- `scripts/init/py-package.inc`: Removed `$?` checks
- `scripts/init/py-package.inc`: Removed `$?` checks

### Files Added
- `scripts/init/patches/verilator.patch`: Renamed from `scripts/5222-gnu-20.patch`
- `scripts/init/patches/verilator-patch.md`: Documentation for verilator patch
- `scripts/init/patches/circt.patch`: Renamed from `scripts/circt-disable-esi.patch`
- `scripts/init/patches/circt-patch.md`: Documentation for circt patch
- `scripts/init/patches/ramulator2.patch`: Renamed from `scripts/ramulator2-template.patch`
- `scripts/init/patches/ramulator2-patch.md`: Moved from `scripts/ramulator2-patch.md`

### Files Removed
- `scripts/build-verilator.sh`: Deleted (was already removed in previous TODO)

## Technical Decisions

### Makefile Simplification
The removal of `$?` checks was justified because Makefile automatically fails when commands return non-zero exit codes. The explicit checks were redundant and made the makefiles more complex than necessary.

### Patch Organization
Centralizing patches in `scripts/init/patches/` improves maintainability by:
- Providing a single location for all build patches
- Enabling better documentation and naming conventions
- Making it easier to track which patches are applied to which components

### Documentation Strategy
Each patch now has an associated markdown file explaining its purpose, making it easier for developers to understand why each patch is necessary and what it does.