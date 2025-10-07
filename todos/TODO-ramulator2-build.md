# Goal

1. Make the build script for ramulator2, `../scripts/init/ramulator2.sh`, idempotent.
2. Update the existing patch `../scripts/ramulator2-template.patch` as per the updated document `../scripts/ramulator2-patch.md`.

# Action Items

1. The existing `../scripts/init/ramulator2.sh` is not idempotent because it uses `git apply` to apply a patch, which fails if the patch has already been applied. Modify the script to check if the patch has already been applied before attempting to apply it. If it has been applied, skip the patching step. Alternatively, reset any changes in the `3rd-party/ramulator2` directory before applying the patch to ensure a clean state.
  - This is for you to decide.
  - After making the changes, run `source ../scripts/init/ramulator2.sh` twice in `zsh` to make sure it works.
    - Commit here without verification.
2. Review the document `../scripts/ramulator2-patch.md` to understand the changes that need to be made to the existing patch `../scripts/ramulator2-template.patch`.
  - The write back hook is a TODO in `ramulator2/dram_controller/impl/generic_dram_controller.cpp`, find that and modify it manually.
    - Specifically, you just need to add something like below to update the latency, and push the request to pending so that the callback is also invoked for write requests.
```cpp
} else if (req_it->type_id == Request::Type::Write) {
  // NEW: mirror read handling for writes
  // Prefer a dedicated write latency if your DRAM model exposes it; else reuse read latency for now.
  auto write_lat = m_dram->m_write_latency; // if this field exists in your build
  if (!write_lat) write_lat = m_dram->m_read_latency; // fallback for POC
  req_it->depart = m_clk + write_lat;
  pending.push_back(*req_it);
}
```
  - Make sure it compiles.
  - Pack the changes into a new patch file that overwrites the old `../scripts/ramulator2-template.patch`, commit here without verification!
  - Make sure the new patch can be applied cleanly to a fresh clone of the ramulator2 repository.
3. Add the idempotent ramulator2 build script to the beginning of `scripts/pre-commit` to ensure it checks before every commit.
  - Commit here with verification!

# Checklist

## Completed Changes

1. **Made ramulator2.sh idempotent**: Updated the build script to properly detect when patches are already applied by checking both git reverse apply and file content verification.

2. **Updated ramulator2-template.patch**: Created a new patch file that includes:
   - Added `*.dylib` to `.gitignore` for macOS compatibility
   - Added `template` keyword to `param.h` for C++ standard compliance
   - Implemented write hook in `generic_dram_controller.cpp` to properly handle write request callbacks

3. **Added ramulator2 build check to pre-commit hooks**: The pre-commit script now ensures ramulator2 is built and patched before every commit, providing early detection of build issues.

## Summary

All three action items have been completed successfully:
- ✅ ramulator2.sh is now idempotent and can be run multiple times safely
- ✅ ramulator2-template.patch has been updated with the write hook fix
- ✅ Pre-commit hooks now include ramulator2 build verification

The changes ensure consistent build environment across development and resolve the write transaction issue mentioned in the ramulator2 GitHub issue #30.
